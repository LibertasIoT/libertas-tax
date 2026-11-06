pub(crate) const CORE_STRINGS: &[(&str, &str)] = &[
    ("START_NEW_RETURN", "Start a new 2026 return"),
    (
        "STORED_RETURN_OLDER_VERSION",
        "Your saved return is from an older version and cannot be opened. You can start a new return for this year. Your saved return is kept until you confirm.",
    ),
    (
        "BROKER_CONTRACTS_TREATMENT_NEEDED",
        "Identify whether these contracts have the listed special treatment.",
    ),
    (
        "BROKER_SALE_BASIS_NEEDED",
        "Supply your established basis or explicitly choose the existing conservative zero-basis treatment.",
    ),
    ("BROKER_SALE_USE_NEEDED", "Choose how the sold property was used."),
    ("DISPLAY_BROKER_YEAR", "Year and status"),
    ("DOCUMENT_K1_RESEARCH_CREDIT_LINE", "Your eligibility for this K-1 research credit"),
    (
        "SOCIAL_PREMIUM_REIMBURSEMENT_NEEDED",
        "How much of this statement’s Medicare premiums was reimbursed tax-free from the qualified HSA spending already entered? Enter zero if none.",
    ),
    (
        "SSA_PRINTED_BOXES_DIFFER",
        "The benefit statement's printed box 5 must equal box 3 less box 4. Check the paper and correct the entries.",
    ),
    (
        "DOCUMENT_VEHICLE_REFUND_BENEFIT",
        "How did the refunded vehicle interest affect your prior return?",
    ),
    (
        "EXPLAIN_VEHICLE_INTEREST_RECOVERY",
        "The taxable recovery is the reduction in the prior Schedule 1-A deduction that reduced tax; current-year interest is unchanged.",
    ),
    (
        "DOCUMENT_TRUMP_ACCOUNT_FACTS",
        "Complete this beneficiary’s Trump-account contributions and corrective-return facts on Other taxes.",
    ),
    (
        "R_DESIGNATED_ROTH_YEAR_NEEDED",
        "The designated Roth distribution needs its printed first contribution year to establish the five-year period.",
    ),
    ("DOCUMENT_VEHICLE_ORIGINAL_USE", "Did the vehicle's original use start with you?"),
    ("DOCUMENT_VEHICLE_ASSEMBLY", "Was the vehicle finally assembled in the United States?"),
    (
        "DOCUMENT_QUESTIONS_CHANGED",
        "The questions for these documents changed. Reopen the topic before saving.",
    ),
    (
        "DOCUMENT_SITUATIONS_NEEDED",
        "Answer whether any of the listed situations applies to these documents.",
    ),
    ("DOCUMENT_SITUATION_TWICE", "This situation is already listed for that document."),
    (
        "DOCUMENT_SITUATION_NOT_ON_DOCUMENT",
        "This situation does not apply to that document. Choose a document it applies to.",
    ),
    ("DOCUMENT_K1_DISPOSITION", "Answer what happened to this interest during the year."),
    ("DOCUMENT_K1_PARTNER_RECORDS", "Your partnership records"),
    ("DOCUMENT_LTC_INTEREST", "Interest paid in addition to the benefits"),
    (
        "LTC1099_ILLNESS_UNANSWERED",
        "The accelerated death benefits need the insured person's printed illness status before a complete estimate is available.",
    ),
    (
        "SOCIAL_PREMIUM_USE_NEEDED",
        "Are any Medicare premiums on this statement already included in a self-employed health plan?",
    ),
    (
        "DOCUMENT_K1_OPENING_BASIS",
        "Enter the actual opening basis, including zero when it is zero.",
    ),
    ("DOCUMENT_ESTATE_ROYALTY_CHARACTER", "Source character of the royalties and their deductions"),
    ("DOCUMENT_ESTATE_FINAL_TAX_CHARACTER", "The fiduciary's breakdown of final-year taxes"),
    ("DEBT_TREATMENT_NEEDED", "How is this canceled debt treated?"),
    ("DEBT_PROPERTY_NEEDED", "What property did the creditor actually take?"),
    (
        "DEBT_PROPERTY_FMV_NEEDED",
        "Restore the property's printed fair market value or correct the property answer.",
    ),
    (
        "ESTATE_PARTICIPATION_NEEDED",
        "Complete your participation records for this fiduciary activity.",
    ),
    ("DOCUMENT_K1_PARTICIPATION", "How you took part in this business"),
    ("DOCUMENT_K1_AMT_RECORDS", "Your alternative minimum tax records"),
    ("DOCUMENT_K1_LLC_SELF_EMPLOYMENT", "Does box 14 code A include your share of box 1?"),
    ("DOCUMENT_K1_RENTAL_RECORDS", "Your rental activity records"),
    ("DOCUMENT_K1_RENTAL_PARTICIPATION", "How you took part in the rental real estate"),
    ("DOCUMENT_K1_LIMITED_FOR_SE", "Limited partner for self-employment tax"),
    ("DOCUMENT_K1_DEPLETION_SOURCES", "Depletion adjustment by activity on this K-1"),
    (
        "ISSUE_K1_AMT_LINE_CLASSIFICATION_UNAVAILABLE",
        "The K-1's supporting statement is needed to identify the AMT activity adjustment's form line.",
    ),
    (
        "EXPLAIN_K1_AMT_PASSIVE_ADJUSTMENT",
        "AMT minus regular-tax activity income after the passive loss limits.",
    ),
    (
        "EXPLAIN_K1_AMT_LOSS_LIMIT_ADJUSTMENT",
        "AMT minus regular-tax nonpassive activity income after basis and at-risk loss limits.",
    ),
    (
        "EXPLAIN_K1_AMT_DEPLETION",
        "The signed depletion adjustment after the applicable source and activity limits, counted once in AMT income.",
    ),
    (
        "K1_DEPLETION_SOURCE_UNAVAILABLE",
        "The depletion adjustment needs an identifiable activity on this K-1's supporting statement.",
    ),
    (
        "K1_DEPLETION_SOURCE_NEEDED",
        "Enter each activity's signed depletion adjustment from this K-1's supporting statement.",
    ),
    ("K1_DEPLETION_SOURCE_INVALID", "Choose each actual activity on this K-1 only once."),
    (
        "K1_DEPLETION_SOURCE_TOTAL",
        "The activity adjustments must add to the printed code C total, including offsetting positive and negative amounts.",
    ),
    ("R_TRUMP_BASIS_NEEDED", "Enter the trustee-reported basis of the acquired Trump account."),
    ("R_TAXABLE_METHOD_NEEDED", "Choose how to figure this distribution's taxable amount."),
    ("R_TREATMENT_NEEDED", "Answer what the printed distribution codes mean for you."),
    (
        "DIRECT_PARTICIPATION_OTHER_ACTIVITY_SOURCE_REQUIRED",
        "Establish the qualifying hours and legal activity grouping of the other trades or businesses before applying the combined significant-participation test.",
    ),
    (
        "RENTAL_FOREIGN_RECOVERY_HISTORY_UNAVAILABLE",
        "This foreign rental needs its actual original ADS election, change-of-use or disposition history before its depreciation can be calculated.",
    ),
    (
        "EXPLAIN_RENTAL_HOME_THIRD_TIER_ALLOWED",
        "Remaining rental income allows current casualty losses first, then current depreciation, then prior casualty and depreciation carryovers.",
    ),
    (
        "EXPLAIN_RENTAL_HOME_CASUALTY_ALLOWED",
        "The home-use worksheet retains casualty character for current losses and prior carryovers. Its allowed casualty is deducted once through Form 4684 and Form 4797.",
    ),
    (
        "EXPLAIN_RENTAL_HOME_CASUALTY_CARRY",
        "The actual casualty part still carried to the next home-use worksheet.",
    ),
    ("LINE_AMT_RENTAL_N_HOME_THIRD_TIER_ALLOWED", "AMT rental {0}: allowed third-tier expenses"),
    (
        "EXPLAIN_RENTAL_HOME_CASUALTY_OTHERWISE",
        "Worksheet 5-1 line 2c is the rental portion of this dwelling's otherwise deductible personal casualty loss. The shared Form 4684 worksheet uses the whole saved property once; itemizing includes ordinary and qualified disaster allowances, while an increased standard deduction includes only the qualified allowance. This amount joins the allowed excess casualty on Section B, not Schedule E depreciation.",
    ),
    (
        "LINE_RENTAL_N_HOME_CASUALTY_OTHERWISE",
        "Rental {0}: otherwise deductible casualty (Worksheet 5-1 line 2c)",
    ),
    (
        "LINE_AMT_RENTAL_N_HOME_CASUALTY_OTHERWISE",
        "AMT rental {0}: otherwise deductible casualty (Worksheet 5-1 line 2c)",
    ),
    ("LINE_AMT_RENTAL_N_HOME_CASUALTY_ALLOWED", "AMT rental {0}: allowed home-use casualty"),
    ("LINE_AMT_RENTAL_N_HOME_CASUALTY_CARRYOVER", "AMT rental {0}: casualty carryover"),
    ("LINE_RENTAL_N_HOME_THIRD_TIER_ALLOWED", "Rental {0}: allowed third-tier expenses"),
    ("LINE_RENTAL_N_HOME_CASUALTY_ALLOWED", "Rental {0}: allowed home-use casualty"),
    ("LINE_RENTAL_N_HOME_CASUALTY_CARRYOVER", "Rental {0}: casualty carryover"),
    ("LINE_RENTAL_HOME_PRIOR_CASUALTY", "Prior rental home-use casualty allowed"),
    ("LINE_RENTAL_HOME_PRIOR_CASUALTY_ORDINARY", "Prior rental home-use casualty loss"),
    ("LINE_RENTAL_N_ENERGY_DEDUCTION", "Rental {0}: Form 7205 energy deduction"),
    (
        "EXPLAIN_RENTAL_ENERGY_DEDUCTION",
        "The documented physical-building allowance and actual qualifying cost limit the section 179D deduction. This same source-cent amount reduces the qualifying property's depreciation basis; it is already included in Schedule E other expenses.",
    ),
    (
        "RENTAL_ENERGY_BUILDING_RECORDS",
        "Review the physical building designation, positive conditioned area and documented ownership fraction.",
    ),
    (
        "RENTAL_ENERGY_HISTORY_RECORDS",
        "Enter each documentary year once, identifying all relevant claimants and its actual nonnegative deduction or absence.",
    ),
    (
        "EXPLAIN_K1_ENTITY_NONDEDUCTIBLE_USED",
        "These are basis reductions, not tax deductions or activity losses.",
    ),
    (
        "EXPLAIN_PARTNER_ENTITY_NONDEDUCTIBLE_CARRY",
        "The partnership basis worksheet retains expenses exceeding available basis separately from deductible losses.",
    ),
    (
        "EXPLAIN_SCORP_ENTITY_NONDEDUCTIBLE_CARRY",
        "Unused expenses carry only under the established continuing losses-first election.",
    ),
    (
        "TIMBER_TOTAL_UNREPRESENTABLE",
        "The combined original amortization sources exceed the native monetary result range. Correct the source amounts before a complete estimate can be calculated.",
    ),
    (
        "ISSUE_AMORTIZATION_DISPOSITION_SOURCE_NEEDED",
        "Resolve this cost cohort's original election, calendar, credit adjustment and actual prior recovery before the complete estimate can be calculated.",
    ),
    (
        "ISSUE_RESEARCH_DEVELOPED_PROPERTY_TRANSITION",
        "Elective or pre-2022 research converted to developed property needs its actual section 167 basis-transfer treatment before the complete estimate can be calculated.",
    ),
    ("LINE_BUSINESS_N_AMORTIZATION_N_DESCRIPTION", "Business {0}, cost {1}: Cost description"),
    (
        "LINE_BUSINESS_N_AMORTIZATION_N_STARTED",
        "Business {0}, cost {1}: Original amortization start",
    ),
    ("LINE_BUSINESS_N_AMORTIZATION_N_BASIS", "Business {0}, cost {1}: Amortizable amount"),
    ("LINE_BUSINESS_N_AMORTIZATION_N_CODE", "Business {0}, cost {1}: Code section or procedure"),
    ("LINE_BUSINESS_N_AMORTIZATION_N_PERIOD", "Business {0}, cost {1}: Recovery months"),
    ("LINE_BUSINESS_N_AMORTIZATION_N_DEDUCTION", "Business {0}, cost {1}: Annual amortization"),
    ("LINE_BUSINESS_N_AMORTIZATION_N_CURRENT_EXPENSE", "Business {0}, cost {1}: Current expense"),
    (
        "LINE_BUSINESS_N_AMORTIZATION_N_REMAINING",
        "Business {0}, cost {1}: Remaining amortizable basis",
    ),
    (
        "LINE_BUSINESS_N_AMORTIZATION_N_ORDINARY",
        "Business {0}, cost {1}: Creative-property ordinary disposition income",
    ),
    (
        "LINE_BUSINESS_N_AMORTIZATION_N_DISPOSAL_REALIZED",
        "Business {0} amortized property {1} realized",
    ),
    ("LINE_BUSINESS_N_AMORTIZATION_N_DISPOSAL_COST", "Business {0} amortized property {1} cost"),
    (
        "LINE_BUSINESS_N_AMORTIZATION_N_DISPOSAL_RECOVERY",
        "Business {0} amortized property {1} recovery",
    ),
    (
        "LINE_BUSINESS_N_AMORTIZATION_N_DISPOSAL_ADJUSTED_BASIS",
        "Business {0} amortized property {1} adjusted basis",
    ),
    (
        "LINE_BUSINESS_N_AMORTIZATION_N_DISPOSAL_ORDINARY",
        "Business {0} amortized property {1} ordinary",
    ),
    (
        "LINE_BUSINESS_N_AMORTIZATION_N_DISPOSAL_SECTION1231",
        "Business {0} amortized property {1} section 1231",
    ),
    (
        "LINE_BUSINESS_N_AMORTIZATION_N_DISPOSAL_POSTPONED",
        "Business {0} amortized property {1} postponed",
    ),
    (
        "LINE_BUSINESS_N_AMORTIZATION_N_DISPOSAL_DISALLOWED",
        "Business {0} amortized property {1} disallowed",
    ),
    (
        "LINE_BUSINESS_N_AMORTIZATION_N_DISPOSAL_FORM6252_ADJUSTED_BASIS",
        "Business {0} amortized property {1} disposal Form 6252 adjusted basis",
    ),
    (
        "LINE_BUSINESS_N_AMORTIZATION_N_DISPOSAL_FORM6252_ASSUMED_DEBT",
        "Business {0} amortized property {1} disposal Form 6252 assumed debt",
    ),
    (
        "LINE_BUSINESS_N_AMORTIZATION_N_DISPOSAL_FORM6252_CONTRACT_PRICE",
        "Business {0} amortized property {1} disposal Form 6252 contract price",
    ),
    (
        "LINE_BUSINESS_N_AMORTIZATION_N_DISPOSAL_FORM6252_COST",
        "Business {0} amortized property {1} disposal Form 6252 cost",
    ),
    (
        "LINE_BUSINESS_N_AMORTIZATION_N_DISPOSAL_FORM6252_DEEMED_PAYMENT",
        "Business {0} amortized property {1} disposal Form 6252 deemed payment",
    ),
    (
        "LINE_BUSINESS_N_AMORTIZATION_N_DISPOSAL_FORM6252_DEPRECIATION",
        "Business {0} amortized property {1} disposal Form 6252 depreciation",
    ),
    (
        "LINE_BUSINESS_N_AMORTIZATION_N_DISPOSAL_FORM6252_EXCESS_DEBT",
        "Business {0} amortized property {1} disposal Form 6252 excess debt",
    ),
    (
        "LINE_BUSINESS_N_AMORTIZATION_N_DISPOSAL_FORM6252_EXPENSES",
        "Business {0} amortized property {1} disposal Form 6252 expenses",
    ),
    (
        "LINE_BUSINESS_N_AMORTIZATION_N_DISPOSAL_FORM6252_GAIN",
        "Business {0} amortized property {1} disposal Form 6252 gain",
    ),
    (
        "LINE_BUSINESS_N_AMORTIZATION_N_DISPOSAL_FORM6252_GROSS_PROFIT",
        "Business {0} amortized property {1} disposal Form 6252 gross profit",
    ),
    (
        "LINE_BUSINESS_N_AMORTIZATION_N_DISPOSAL_FORM6252_PROFIT",
        "Business {0} amortized property {1} disposal Form 6252 profit before exclusion",
    ),
    (
        "LINE_BUSINESS_N_AMORTIZATION_N_DISPOSAL_FORM6252_PERCENTAGE",
        "Business {0} amortized property {1} disposal Form 6252 gross profit percentage",
    ),
    (
        "LINE_BUSINESS_N_AMORTIZATION_N_DISPOSAL_FORM6252_INCOME",
        "Business {0} amortized property {1} disposal Form 6252 income",
    ),
    (
        "LINE_BUSINESS_N_AMORTIZATION_N_DISPOSAL_FORM6252_NET_PRICE",
        "Business {0} amortized property {1} disposal Form 6252 net price",
    ),
    (
        "LINE_BUSINESS_N_AMORTIZATION_N_DISPOSAL_FORM6252_ORDINARY",
        "Business {0} amortized property {1} disposal Form 6252 ordinary",
    ),
    (
        "LINE_BUSINESS_N_AMORTIZATION_N_DISPOSAL_FORM6252_PAYMENTS",
        "Business {0} amortized property {1} disposal Form 6252 payments",
    ),
    (
        "LINE_BUSINESS_N_AMORTIZATION_N_DISPOSAL_FORM6252_PRICE",
        "Business {0} amortized property {1} disposal Form 6252 price",
    ),
    (
        "LINE_BUSINESS_N_AMORTIZATION_N_DISPOSAL_FORM6252_RECAPTURE",
        "Business {0} amortized property {1} disposal Form 6252 recapture",
    ),
    (
        "LINE_BUSINESS_N_AMORTIZATION_N_DISPOSAL_FORM6252_TOTAL",
        "Business {0} amortized property {1} disposal Form 6252 total",
    ),
    (
        "LINE_BUSINESS_N_AMORTIZATION_N_DISPOSAL_FORM6252_TOTAL_PAYMENTS",
        "Business {0} amortized property {1} disposal Form 6252 total payments",
    ),
    (
        "LINE_BUSINESS_N_AMORTIZATION_N_DISPOSAL_FORM6252_UNRECAPTURED_N",
        "Business {0} amortized property {1} disposal Form 6252 unrecaptured {2}",
    ),
    ("LINE_BUSINESS_N_TIMBER_N_DISPOSAL_REALIZED", "Business {0} timber property {1} realized"),
    ("LINE_BUSINESS_N_TIMBER_N_DISPOSAL_COST", "Business {0} timber property {1} cost"),
    ("LINE_BUSINESS_N_TIMBER_N_DISPOSAL_RECOVERY", "Business {0} timber property {1} recovery"),
    (
        "LINE_BUSINESS_N_TIMBER_N_DISPOSAL_ADJUSTED_BASIS",
        "Business {0} timber property {1} adjusted basis",
    ),
    ("LINE_BUSINESS_N_TIMBER_N_DISPOSAL_ORDINARY", "Business {0} timber property {1} ordinary"),
    (
        "LINE_BUSINESS_N_TIMBER_N_DISPOSAL_SECTION1231",
        "Business {0} timber property {1} section 1231",
    ),
    ("LINE_BUSINESS_N_TIMBER_N_DISPOSAL_POSTPONED", "Business {0} timber property {1} postponed"),
    ("LINE_BUSINESS_N_TIMBER_N_DISPOSAL_DISALLOWED", "Business {0} timber property {1} disallowed"),
    (
        "LINE_BUSINESS_N_TIMBER_N_DISPOSAL_FORM6252_ADJUSTED_BASIS",
        "Business {0} timber property {1} disposal Form 6252 adjusted basis",
    ),
    (
        "LINE_BUSINESS_N_TIMBER_N_DISPOSAL_FORM6252_ASSUMED_DEBT",
        "Business {0} timber property {1} disposal Form 6252 assumed debt",
    ),
    (
        "LINE_BUSINESS_N_TIMBER_N_DISPOSAL_FORM6252_CONTRACT_PRICE",
        "Business {0} timber property {1} disposal Form 6252 contract price",
    ),
    (
        "LINE_BUSINESS_N_TIMBER_N_DISPOSAL_FORM6252_COST",
        "Business {0} timber property {1} disposal Form 6252 cost",
    ),
    (
        "LINE_BUSINESS_N_TIMBER_N_DISPOSAL_FORM6252_DEEMED_PAYMENT",
        "Business {0} timber property {1} disposal Form 6252 deemed payment",
    ),
    (
        "LINE_BUSINESS_N_TIMBER_N_DISPOSAL_FORM6252_DEPRECIATION",
        "Business {0} timber property {1} disposal Form 6252 depreciation",
    ),
    (
        "LINE_BUSINESS_N_TIMBER_N_DISPOSAL_FORM6252_EXCESS_DEBT",
        "Business {0} timber property {1} disposal Form 6252 excess debt",
    ),
    (
        "LINE_BUSINESS_N_TIMBER_N_DISPOSAL_FORM6252_EXPENSES",
        "Business {0} timber property {1} disposal Form 6252 expenses",
    ),
    (
        "LINE_BUSINESS_N_TIMBER_N_DISPOSAL_FORM6252_GAIN",
        "Business {0} timber property {1} disposal Form 6252 gain",
    ),
    (
        "LINE_BUSINESS_N_TIMBER_N_DISPOSAL_FORM6252_GROSS_PROFIT",
        "Business {0} timber property {1} disposal Form 6252 gross profit",
    ),
    (
        "LINE_BUSINESS_N_TIMBER_N_DISPOSAL_FORM6252_PROFIT",
        "Business {0} timber property {1} disposal Form 6252 profit before exclusion",
    ),
    (
        "LINE_BUSINESS_N_TIMBER_N_DISPOSAL_FORM6252_PERCENTAGE",
        "Business {0} timber property {1} disposal Form 6252 gross profit percentage",
    ),
    (
        "LINE_BUSINESS_N_TIMBER_N_DISPOSAL_FORM6252_INCOME",
        "Business {0} timber property {1} disposal Form 6252 income",
    ),
    (
        "LINE_BUSINESS_N_TIMBER_N_DISPOSAL_FORM6252_NET_PRICE",
        "Business {0} timber property {1} disposal Form 6252 net price",
    ),
    (
        "LINE_BUSINESS_N_TIMBER_N_DISPOSAL_FORM6252_ORDINARY",
        "Business {0} timber property {1} disposal Form 6252 ordinary",
    ),
    (
        "LINE_BUSINESS_N_TIMBER_N_DISPOSAL_FORM6252_PAYMENTS",
        "Business {0} timber property {1} disposal Form 6252 payments",
    ),
    (
        "LINE_BUSINESS_N_TIMBER_N_DISPOSAL_FORM6252_PRICE",
        "Business {0} timber property {1} disposal Form 6252 price",
    ),
    (
        "LINE_BUSINESS_N_TIMBER_N_DISPOSAL_FORM6252_RECAPTURE",
        "Business {0} timber property {1} disposal Form 6252 recapture",
    ),
    (
        "LINE_BUSINESS_N_TIMBER_N_DISPOSAL_FORM6252_TOTAL",
        "Business {0} timber property {1} disposal Form 6252 total",
    ),
    (
        "LINE_BUSINESS_N_TIMBER_N_DISPOSAL_FORM6252_TOTAL_PAYMENTS",
        "Business {0} timber property {1} disposal Form 6252 total payments",
    ),
    (
        "LINE_BUSINESS_N_TIMBER_N_DISPOSAL_FORM6252_UNRECAPTURED_N",
        "Business {0} timber property {1} disposal Form 6252 unrecaptured {2}",
    ),
    (
        "AMORTIZATION_TOTAL_UNREPRESENTABLE",
        "Correct the original amortization sources before their combined amount can be calculated.",
    ),
    (
        "LINE_BUSINESS_N_AMORTIZATION_N_DISPOSAL_GAIN",
        "Business {0} amortized property {1} realized gain",
    ),
    ("LINE_BUSINESS_N_TIMBER_N_DISPOSAL_GAIN", "Business {0} timber property {1} realized gain"),
    ("EXPLAIN_OTHER_AMORTIZATION_BASIS", "Original amortizable basis"),
    ("EXPLAIN_OTHER_AMORTIZATION_PERIOD", "Original recovery period in months"),
    ("EXPLAIN_OTHER_AMORTIZATION_DEDUCTION", "Current amortization deduction"),
    ("EXPLAIN_OTHER_AMORTIZATION_CURRENT_EXPENSE", "Current expense deduction"),
    ("EXPLAIN_OTHER_AMORTIZATION_REMAINING", "Remaining amortizable basis"),
    (
        "EXPLAIN_CREATIVE_PROPERTY_ORDINARY",
        "Creative property disposition proceeds, ordinary property income",
    ),
    ("LINE_BUSINESS_N_TIMBER_N_DESCRIPTION", "Business {0} timber cohort {1} description"),
    (
        "LINE_BUSINESS_N_TIMBER_N_STARTED",
        "Business {0} timber cohort {1} original amortization start",
    ),
    ("LINE_BUSINESS_N_TIMBER_N_CODE", "Business {0} timber cohort {1} Code section"),
    ("LINE_BUSINESS_N_TIMBER_N_BASIS", "Business {0} timber cohort {1} elected amortizable basis"),
    ("LINE_BUSINESS_N_TIMBER_N_PERIOD", "Business {0} timber cohort {1} amortization period"),
    ("LINE_BUSINESS_N_TIMBER_N_AMORTIZATION", "Business {0} timber cohort {1} amortization"),
    ("LINE_BUSINESS_N_TIMBER_N_CURRENT_EXPENSE", "Business {0} timber cohort {1} current expense"),
    (
        "LINE_BUSINESS_N_TIMBER_N_REMAINING",
        "Business {0} timber cohort {1} remaining capital basis",
    ),
    (
        "EXPLAIN_TIMBER_ELECTED_BASIS",
        "Original section 194(a) basis after actually excluded reimbursements and the separate original expense election. Unelected capital costs are retained, not automatically amortized.",
    ),
    (
        "EXPLAIN_TIMBER_PERIOD",
        "Section 194(a): 84 months beginning in July of the original expenditure year.",
    ),
    (
        "EXPLAIN_TIMBER_AMORTIZATION",
        "Original elected basis times the current allowed months over 84, rounded once and capped at actual remaining elected basis. Ordinary disposition includes its month; the original period never restarts.",
    ),
    (
        "EXPLAIN_TIMBER_CURRENT_EXPENSE",
        "Actual original section 194(b) expense election, deducted only in its expenditure year. Other-source elections only use the property limit and are not deducted again here.",
    ),
    (
        "EXPLAIN_TIMBER_REMAINING",
        "Qualified capital basis after the original expense election, actual prior amortization and this year's allowance, including unelected capital basis.",
    ),
    ("LINE_HOME_OFFICE_CASUALTY_ORDINARY", "Business casualty loss from home use"),
    ("LINE_HOME_N_OWNER_N_FORM_N_ALLOWED", "Home {0} ({1}) Form {2} allowed"),
    ("LINE_HOME_N_OWNER_N_FORM_N_BASIS", "Home {0} ({1}) Form {2} basis"),
    ("LINE_HOME_N_OWNER_N_FORM_N_BUILDING", "Home {0} ({1}) Form {2} building"),
    ("LINE_HOME_N_OWNER_N_FORM_N_BUSINESS_BASIS", "Home {0} ({1}) Form {2} business basis"),
    ("LINE_HOME_N_OWNER_N_FORM_N_CASUALTY", "Home {0} ({1}) Form {2} casualty routed separately"),
    (
        "LINE_HOME_N_OWNER_N_FORM_N_CASUALTY_OTHERWISE_ALLOWED",
        "Home {0} ({1}) Form {2} otherwise allowable casualty",
    ),
    (
        "LINE_HOME_N_OWNER_N_FORM_N_DEDUCTIBLE_DIRECT",
        "Home {0} ({1}) Form {2} deductible direct expenses",
    ),
    (
        "LINE_HOME_N_OWNER_N_FORM_N_DEDUCTIBLE_INDIRECT",
        "Home {0} ({1}) Form {2} deductible indirect expenses",
    ),
    ("LINE_HOME_N_OWNER_N_FORM_N_DEPRECIATION", "Home {0} ({1}) Form {2} depreciation"),
    (
        "LINE_HOME_N_OWNER_N_FORM_N_DEPRECIATION_ALLOWABLE",
        "Home {0} ({1}) Form {2} depreciation allowable",
    ),
    (
        "LINE_HOME_N_OWNER_N_FORM_N_DEPRECIATION_ALLOWED",
        "Home {0} ({1}) Form {2} depreciation allowed",
    ),
    (
        "LINE_HOME_N_OWNER_N_FORM_N_DEPRECIATION_CARRYOVER",
        "Home {0} ({1}) Form {2} depreciation carryover",
    ),
    ("LINE_HOME_N_OWNER_N_FORM_N_DEPRECIATION_LIMIT", "Home {0} ({1}) Form {2} depreciation limit"),
    ("LINE_HOME_N_OWNER_N_FORM_N_DEPRECIATION_TOTAL", "Home {0} ({1}) Form {2} depreciation total"),
    ("LINE_HOME_N_OWNER_N_FORM_N_DIRECT_REPAIRS", "Home {0} ({1}) Form {2} direct repairs"),
    ("LINE_HOME_N_OWNER_N_FORM_N_DIRECT_TOTAL", "Home {0} ({1}) Form {2} direct total"),
    ("LINE_HOME_N_OWNER_N_FORM_N_EXCESS_CASUALTY", "Home {0} ({1}) Form {2} excess casualty"),
    ("LINE_HOME_N_OWNER_N_FORM_N_EXCESS_INTEREST", "Home {0} ({1}) Form {2} excess interest"),
    ("LINE_HOME_N_OWNER_N_FORM_N_EXCESS_TAXES", "Home {0} ({1}) Form {2} excess taxes"),
    ("LINE_HOME_N_OWNER_N_FORM_N_INCOME", "Home {0} ({1}) Form {2} income"),
    ("LINE_HOME_N_OWNER_N_FORM_N_INDIRECT_SHARE", "Home {0} ({1}) Form {2} indirect share"),
    ("LINE_HOME_N_OWNER_N_FORM_N_INDIRECT_TOTAL", "Home {0} ({1}) Form {2} indirect total"),
    ("LINE_HOME_N_OWNER_N_FORM_N_INSURANCE", "Home {0} ({1}) Form {2} insurance"),
    ("LINE_HOME_N_OWNER_N_FORM_N_LAND", "Home {0} ({1}) Form {2} land"),
    ("LINE_HOME_N_OWNER_N_FORM_N_LIMIT", "Home {0} ({1}) Form {2} limit"),
    ("LINE_HOME_N_OWNER_N_FORM_N_MORTGAGE_INTEREST", "Home {0} ({1}) Form {2} mortgage interest"),
    ("LINE_HOME_N_OWNER_N_FORM_N_OPERATING_ALLOWED", "Home {0} ({1}) Form {2} operating allowed"),
    (
        "LINE_HOME_N_OWNER_N_FORM_N_OPERATING_CARRYOVER",
        "Home {0} ({1}) Form {2} operating carryover",
    ),
    ("LINE_HOME_N_OWNER_N_FORM_N_OPERATING_SHARE", "Home {0} ({1}) Form {2} operating share"),
    ("LINE_HOME_N_OWNER_N_FORM_N_OPERATING_TOTAL", "Home {0} ({1}) Form {2} operating total"),
    ("LINE_HOME_N_OWNER_N_FORM_N_OTHER", "Home {0} ({1}) Form {2} other"),
    ("LINE_HOME_N_OWNER_N_FORM_N_PRIOR_DEPRECIATION", "Home {0} ({1}) Form {2} prior depreciation"),
    ("LINE_HOME_N_OWNER_N_FORM_N_PRIOR_OPERATING", "Home {0} ({1}) Form {2} prior operating"),
    ("LINE_HOME_N_OWNER_N_FORM_N_REAL_ESTATE_TAXES", "Home {0} ({1}) Form {2} real estate taxes"),
    ("LINE_HOME_N_OWNER_N_FORM_N_RENT", "Home {0} ({1}) Form {2} rent"),
    ("LINE_HOME_N_OWNER_N_FORM_N_REPAIRS", "Home {0} ({1}) Form {2} repairs"),
    ("LINE_HOME_N_OWNER_N_FORM_N_TOTAL", "Home {0} ({1}) Form {2} total home expenses allowed"),
    ("LINE_HOME_N_OWNER_N_FORM_N_UNLIMITED", "Home {0} ({1}) Form {2} unlimited"),
    ("LINE_HOME_N_OWNER_N_FORM_N_UTILITIES", "Home {0} ({1}) Form {2} utilities"),
    (
        "LINE_HOME_N_OWNER_N_FORM_N_WORKSHEET_BUSINESS_TAXES",
        "Home {0} ({1}) Form {2} worksheet business taxes",
    ),
    ("LINE_HOME_N_OWNER_N_WORKSHEET_DEDUCTION", "Home {0} ({1}) home-use deduction"),
    ("LINE_HOME_N_OWNER_N_WORKSHEET_OPERATING_CARRYOVER", "Home {0} ({1}) operating carryover"),
    (
        "LINE_HOME_N_OWNER_N_WORKSHEET_DEPRECIATION_CARRYOVER",
        "Home {0} ({1}) depreciation and casualty carryover",
    ),
    (
        "HOME_CONTEXT_REQUIRED",
        "This home calculation needs the current return invocation and its observed income and tax limits. A first trial is not a completed return.",
    ),
    (
        "HOME_OUTPUT_UNAVAILABLE",
        "These home source amounts cannot be represented by the native worksheet. No deduction or carryover is inferred from an overflow.",
    ),
    (
        "EXPLAIN_CANONICAL_HOME_WORKSHEET",
        "The home worksheet uses the saved property's costs, this owner's qualifying use, and the linked activities' income and expenses. Casualty amounts go to Form 4684 separately from the business home-use deduction.",
    ),
    ("SECTION_HOME_OFFICES", "Business use of your homes"),
    ("LINE_BUSINESS_CREDIT_CREDIT_CURRENT_YEAR", "Business credit {0} 2026"),
    (
        "LINE_BUSINESS_CREDIT_CREDIT_CURRENT_YEAR_CARRYBACK_N",
        "Business credit {0} 2026 carryback {1}",
    ),
    ("LINE_BUSINESS_CREDIT_CREDIT_CURRENT_YEAR_EXPIRED", "Business credit {0} 2026 expired"),
    ("LINE_BUSINESS_CREDIT_CREDIT_CURRENT_YEAR_PASSIVE", "Business credit {0} 2026 passive"),
    (
        "LINE_BUSINESS_CREDIT_CREDIT_CURRENT_YEAR_PASSIVE_ALLOWED",
        "Business credit {0} 2026 passive allowed",
    ),
    (
        "LINE_BUSINESS_CREDIT_CREDIT_CURRENT_YEAR_PASSIVE_TO_N",
        "Business credit {0} 2026 passive to {1}",
    ),
    (
        "LINE_BUSINESS_CREDIT_CREDIT_CURRENT_YEAR_PASSIVE_TO_NEXT_YEAR",
        "Business credit {0} 2026 passive to 2027",
    ),
    ("LINE_BUSINESS_CREDIT_CREDIT_CURRENT_YEAR_TO_N", "Business credit {0} 2026 to {1}"),
    ("LINE_BUSINESS_CREDIT_CREDIT_CURRENT_YEAR_TO_NEXT_YEAR", "Business credit {0} 2026 to 2027"),
    ("LINE_BUSINESS_CREDIT_CREDIT_CURRENT_YEAR_USED", "Business credit {0} 2026 used"),
    (
        "LINE_BUSINESS_CREDIT_CREDIT_N_PASSIVE_TO_NEXT_YEAR",
        "Business credit {0} {1} passive to 2027",
    ),
    ("LINE_BUSINESS_CREDIT_CREDIT_N_TO_NEXT_YEAR", "Business credit {0} {1} to 2027"),
    (
        "LINE_BUSINESS_CREDIT_CREDIT_SOURCE_N_FILER_N_CURRENT_YEAR",
        "{0}, source {1}, {2}, 2026 credit",
    ),
    (
        "LINE_BUSINESS_CREDIT_CREDIT_SOURCE_N_FILER_N_CURRENT_YEAR_CARRYBACK_N",
        "{0}, source {1}, {2}, 2026 credit carried back to {3}",
    ),
    (
        "LINE_BUSINESS_CREDIT_CREDIT_SOURCE_N_FILER_N_CURRENT_YEAR_EXPIRED",
        "{0}, source {1}, {2}, 2026 credit expired",
    ),
    (
        "LINE_BUSINESS_CREDIT_CREDIT_SOURCE_N_FILER_N_CURRENT_YEAR_PASSIVE",
        "{0}, source {1}, {2}, 2026 passive credit",
    ),
    (
        "LINE_BUSINESS_CREDIT_CREDIT_SOURCE_N_FILER_N_CURRENT_YEAR_PASSIVE_ALLOWED",
        "{0}, source {1}, {2}, 2026 passive credit allowed",
    ),
    (
        "LINE_BUSINESS_CREDIT_CREDIT_SOURCE_N_FILER_N_CURRENT_YEAR_PASSIVE_TO_N",
        "{0}, source {1}, {2}, 2026 passive credit carried to {3}",
    ),
    (
        "LINE_BUSINESS_CREDIT_CREDIT_SOURCE_N_FILER_N_CURRENT_YEAR_PASSIVE_TO_NEXT_YEAR",
        "{0}, source {1}, {2}, 2026 passive credit carried to 2027",
    ),
    (
        "LINE_BUSINESS_CREDIT_CREDIT_SOURCE_N_FILER_N_CURRENT_YEAR_TO_N",
        "{0}, source {1}, {2}, 2026 credit carried to {3}",
    ),
    (
        "LINE_BUSINESS_CREDIT_CREDIT_SOURCE_N_FILER_N_CURRENT_YEAR_TO_NEXT_YEAR",
        "{0}, source {1}, {2}, 2026 credit carried to 2027",
    ),
    (
        "LINE_BUSINESS_CREDIT_CREDIT_SOURCE_N_FILER_N_CURRENT_YEAR_USED",
        "{0}, source {1}, {2}, 2026 credit used",
    ),
    (
        "LINE_BUSINESS_CREDIT_CREDIT_SOURCE_N_FILER_N_N_PASSIVE_TO_NEXT_YEAR",
        "{0}, source {1}, {2}, {3} passive credit carried to 2027",
    ),
    (
        "LINE_BUSINESS_CREDIT_CREDIT_SOURCE_N_FILER_N_N_TO_NEXT_YEAR",
        "{0}, source {1}, {2}, {3} credit carried to 2027",
    ),
    ("LINE_CLEAN_VEHICLE_MODIFIED_AGI_CURRENT_YEAR", "Clean vehicle modified AGI 2026"),
    ("LINE_CLEAN_VEHICLE_MODIFIED_AGI_PRIOR_YEAR", "Clean vehicle modified AGI 2025"),
    ("LINE_FARMING_FISHING_GROSS_INCOME_RETURN_YEAR", "Farming fishing gross income 2026"),
    ("LINE_FARM_N_CROP_INSURANCE_DEFERRED_NEXT_YEAR", "Farm {0} crop insurance deferred 2027"),
    ("LINE_GROSS_INCOME_RETURN_YEAR", "Gross income 2026"),
    ("LINE_TIP_FICA_CREDIT_CARRYFORWARD_TO_NEXT_YEAR", "Tip FICA credit carryforward 2027"),
    (
        "LINE_TIP_FICA_CURRENT_CREDIT_CARRYBACK_TO_PRIOR_YEAR",
        "Tip FICA current credit carryback 2025",
    ),
    (
        "LINE_TIP_FICA_CURRENT_CREDIT_CARRYFORWARD_TO_NEXT_YEAR",
        "Tip FICA current credit carryforward 2027",
    ),
    ("LINE_TIP_FICA_ORIGIN_PRIOR_YEAR_CARRYBACK_USED", "Tip FICA 2024 carryback used"),
    ("LINE_TIP_FICA_PRIOR_YEAR_CREDIT_ORIGIN", "Tip FICA 2025 credit origin"),
    ("LINE_TIP_FICA_PRIOR_YEAR_CREDIT_USED", "Tip FICA 2025 credit used"),
    (
        "WARN_QBI_PARTIAL_COOP_WAGES_LEFT_OUT",
        "{0}: The partial payroll wage quantity does not establish a cooperative patron allocation. QBI wage capacity is left out; payroll expenses and employer credits are retained.",
    ),
    (
        "QBI_AGGREGATION_SOURCES",
        "Review the aggregation sources and annual disclosures. Choose each actual saved business once and keep the current members and source information.",
    ),
    (
        "QBI_AGGREGATION_ACCEPT_SOURCE_FIRST",
        "Save this new income source with no aggregation group, then reopen it before selecting a group.",
    ),
    ("ASSET_USE_ABOVE_TOTAL", "Use for the stated purpose cannot exceed all comparable use."),
    (
        "QPP_RECORD_INVALID",
        "Check this production property's original text, dates, basis and changed-portion records.",
    ),
    (
        "QPP_DESIGNATION_ABOVE_ELIGIBLE",
        "The designated production-property basis cannot exceed the eligible allocable basis.",
    ),
    (
        "QPP_ELECTION_NO_DESIGNATION",
        "Enter the positive basis designated by the production-property election.",
    ),
    (
        "QPP_ELECTION_PROPERTY_FACTS",
        "A production-property election requires domestic nonresidential section 1250 property used directly in the qualifying operation.",
    ),
    (
        "QPP_ELECTION_PRODUCTION_FACTS",
        "A production-property election requires a qualifying production operation, product and substantial transformation.",
    ),
    (
        "QPP_ORIGINAL_OR_ACQUISITION_FACTS",
        "Supply the original-use construction facts or the qualifying used-acquisition facts for this production-property election.",
    ),
    (
        "QPP_CONTRACT_AFTER_ACQUISITION",
        "The binding purchase contract cannot follow the acquisition date.",
    ),
    ("LINE_CHARITY_CARRY_EXPIRED_AFTER_N", "Charitable carryovers expired after {0}"),
    (
        "LINE_CHARITY_PROPERTY_FIFTY_CURRENT_DEDUCTION",
        "50% public-charity property: current gifts deducted",
    ),
    (
        "LINE_CHARITY_PROPERTY_FIFTY_ALLOWED",
        "50% public-charity property: current and prior gifts deducted",
    ),
    ("LINE_CHARITY_PROPERTY_FIFTY_FLOOR", "50% public-charity property: itemizer floor"),
    (
        "LINE_CHARITY_PROPERTY_FIFTY_N_CARRYFORWARD",
        "50% public-charity property: {0} excess and eligible floor carryforward",
    ),
    (
        "LINE_CHARITY_CARRY_N_PROPERTY_FIFTY_OPENING",
        "50% public-charity property, contribution year {0}: opening balance",
    ),
    (
        "LINE_CHARITY_CARRY_N_PROPERTY_FIFTY_USED",
        "50% public-charity property, contribution year {0}: balance used before the itemizer floor",
    ),
    (
        "LINE_CHARITY_CARRY_N_PROPERTY_FIFTY_DEDUCTION",
        "50% public-charity property, contribution year {0}: balance deducted",
    ),
    (
        "LINE_CHARITY_CARRY_N_PROPERTY_FIFTY_CONSUMED_WITHOUT_DEDUCTION",
        "50% public-charity property, contribution year {0}: balance consumed in a standard-deduction year",
    ),
    (
        "LINE_CHARITY_CARRY_N_PROPERTY_FIFTY_REMAINING",
        "50% public-charity property, contribution year {0}: closing balance",
    ),
    (
        "LINE_CHARITY_CARRY_N_PROPERTY_FIFTY_EXPIRED",
        "50% public-charity property, contribution year {0}: expired balance",
    ),
    (
        "LINE_CHARITY_CARRY_N_PROPERTY_FIFTY_UNUSED",
        "50% public-charity property, contribution year {0}: unused balance",
    ),
    (
        "LINE_CHARITY_CARRY_N_PROPERTY_FIFTY_FLOOR_NOT_CARRIED",
        "50% public-charity property, contribution year {0}: floor consumed without deduction",
    ),
    (
        "LINE_CHARITY_PRIOR_PROPERTY_FIFTY_DEDUCTION",
        "50% public-charity property: prior gifts deducted",
    ),
    (
        "LINE_CHARITY_PRIOR_PROPERTY_FIFTY_CONSUMED_WITHOUT_DEDUCTION",
        "50% public-charity property: prior balances consumed without deduction",
    ),
    (
        "LINE_CHARITY_PRIOR_PROPERTY_FIFTY_FLOOR_NOT_CARRIED",
        "50% public-charity property: prior balances consumed by the floor",
    ),
    (
        "LINE_CHARITY_PRIOR_PROPERTY_FIFTY_REMAINING_N",
        "50% public-charity property: balance carried to {0}",
    ),
    (
        "LINE_CHARITY_PRIOR_PROPERTY_FIFTY_EXPIRED_AFTER_N",
        "50% public-charity property: balance expired after {0}",
    ),
    (
        "LINE_CHARITY_CASH_SIXTY_CURRENT_DEDUCTION",
        "60% public-charity cash: current gifts deducted",
    ),
    (
        "LINE_CHARITY_CASH_SIXTY_ALLOWED",
        "60% public-charity cash: current and prior gifts deducted",
    ),
    ("LINE_CHARITY_CASH_SIXTY_FLOOR", "60% public-charity cash: itemizer floor"),
    (
        "LINE_CHARITY_CASH_SIXTY_N_CARRYFORWARD",
        "60% public-charity cash: {0} excess and eligible floor carryforward",
    ),
    (
        "LINE_CHARITY_CARRY_N_CASH_SIXTY_OPENING",
        "60% public-charity cash, contribution year {0}: opening balance",
    ),
    (
        "LINE_CHARITY_CARRY_N_CASH_SIXTY_USED",
        "60% public-charity cash, contribution year {0}: balance used before the itemizer floor",
    ),
    (
        "LINE_CHARITY_CARRY_N_CASH_SIXTY_DEDUCTION",
        "60% public-charity cash, contribution year {0}: balance deducted",
    ),
    (
        "LINE_CHARITY_CARRY_N_CASH_SIXTY_CONSUMED_WITHOUT_DEDUCTION",
        "60% public-charity cash, contribution year {0}: balance consumed in a standard-deduction year",
    ),
    (
        "LINE_CHARITY_CARRY_N_CASH_SIXTY_REMAINING",
        "60% public-charity cash, contribution year {0}: closing balance",
    ),
    (
        "LINE_CHARITY_CARRY_N_CASH_SIXTY_EXPIRED",
        "60% public-charity cash, contribution year {0}: expired balance",
    ),
    (
        "LINE_CHARITY_CARRY_N_CASH_SIXTY_UNUSED",
        "60% public-charity cash, contribution year {0}: unused balance",
    ),
    (
        "LINE_CHARITY_CARRY_N_CASH_SIXTY_FLOOR_NOT_CARRIED",
        "60% public-charity cash, contribution year {0}: floor consumed without deduction",
    ),
    ("LINE_CHARITY_PRIOR_CASH_SIXTY_DEDUCTION", "60% public-charity cash: prior gifts deducted"),
    (
        "LINE_CHARITY_PRIOR_CASH_SIXTY_CONSUMED_WITHOUT_DEDUCTION",
        "60% public-charity cash: prior balances consumed without deduction",
    ),
    (
        "LINE_CHARITY_PRIOR_CASH_SIXTY_FLOOR_NOT_CARRIED",
        "60% public-charity cash: prior balances consumed by the floor",
    ),
    (
        "LINE_CHARITY_PRIOR_CASH_SIXTY_REMAINING_N",
        "60% public-charity cash: balance carried to {0}",
    ),
    (
        "LINE_CHARITY_PRIOR_CASH_SIXTY_EXPIRED_AFTER_N",
        "60% public-charity cash: balance expired after {0}",
    ),
    (
        "LINE_CHARITY_OTHER_THIRTY_CURRENT_DEDUCTION",
        "30% other-organization gifts: current gifts deducted",
    ),
    (
        "LINE_CHARITY_OTHER_THIRTY_ALLOWED",
        "30% other-organization gifts: current and prior gifts deducted",
    ),
    ("LINE_CHARITY_OTHER_THIRTY_FLOOR", "30% other-organization gifts: itemizer floor"),
    (
        "LINE_CHARITY_OTHER_THIRTY_N_CARRYFORWARD",
        "30% other-organization gifts: {0} excess and eligible floor carryforward",
    ),
    (
        "LINE_CHARITY_CARRY_N_OTHER_THIRTY_OPENING",
        "30% other-organization gifts, contribution year {0}: opening balance",
    ),
    (
        "LINE_CHARITY_CARRY_N_OTHER_THIRTY_USED",
        "30% other-organization gifts, contribution year {0}: balance used before the itemizer floor",
    ),
    (
        "LINE_CHARITY_CARRY_N_OTHER_THIRTY_DEDUCTION",
        "30% other-organization gifts, contribution year {0}: balance deducted",
    ),
    (
        "LINE_CHARITY_CARRY_N_OTHER_THIRTY_CONSUMED_WITHOUT_DEDUCTION",
        "30% other-organization gifts, contribution year {0}: balance consumed in a standard-deduction year",
    ),
    (
        "LINE_CHARITY_CARRY_N_OTHER_THIRTY_REMAINING",
        "30% other-organization gifts, contribution year {0}: closing balance",
    ),
    (
        "LINE_CHARITY_CARRY_N_OTHER_THIRTY_EXPIRED",
        "30% other-organization gifts, contribution year {0}: expired balance",
    ),
    (
        "LINE_CHARITY_CARRY_N_OTHER_THIRTY_UNUSED",
        "30% other-organization gifts, contribution year {0}: unused balance",
    ),
    (
        "LINE_CHARITY_CARRY_N_OTHER_THIRTY_FLOOR_NOT_CARRIED",
        "30% other-organization gifts, contribution year {0}: floor consumed without deduction",
    ),
    (
        "LINE_CHARITY_PRIOR_OTHER_THIRTY_DEDUCTION",
        "30% other-organization gifts: prior gifts deducted",
    ),
    (
        "LINE_CHARITY_PRIOR_OTHER_THIRTY_CONSUMED_WITHOUT_DEDUCTION",
        "30% other-organization gifts: prior balances consumed without deduction",
    ),
    (
        "LINE_CHARITY_PRIOR_OTHER_THIRTY_FLOOR_NOT_CARRIED",
        "30% other-organization gifts: prior balances consumed by the floor",
    ),
    (
        "LINE_CHARITY_PRIOR_OTHER_THIRTY_REMAINING_N",
        "30% other-organization gifts: balance carried to {0}",
    ),
    (
        "LINE_CHARITY_PRIOR_OTHER_THIRTY_EXPIRED_AFTER_N",
        "30% other-organization gifts: balance expired after {0}",
    ),
    (
        "LINE_CHARITY_CAPITAL_GAIN_THIRTY_CURRENT_DEDUCTION",
        "30% appreciated public-charity property: current gifts deducted",
    ),
    (
        "LINE_CHARITY_CAPITAL_GAIN_THIRTY_ALLOWED",
        "30% appreciated public-charity property: current and prior gifts deducted",
    ),
    (
        "LINE_CHARITY_CAPITAL_GAIN_THIRTY_FLOOR",
        "30% appreciated public-charity property: itemizer floor",
    ),
    (
        "LINE_CHARITY_CAPITAL_GAIN_THIRTY_N_CARRYFORWARD",
        "30% appreciated public-charity property: {0} excess and eligible floor carryforward",
    ),
    (
        "LINE_CHARITY_CARRY_N_CAPITAL_GAIN_THIRTY_OPENING",
        "30% appreciated public-charity property, contribution year {0}: opening balance",
    ),
    (
        "LINE_CHARITY_CARRY_N_CAPITAL_GAIN_THIRTY_USED",
        "30% appreciated public-charity property, contribution year {0}: balance used before the itemizer floor",
    ),
    (
        "LINE_CHARITY_CARRY_N_CAPITAL_GAIN_THIRTY_DEDUCTION",
        "30% appreciated public-charity property, contribution year {0}: balance deducted",
    ),
    (
        "LINE_CHARITY_CARRY_N_CAPITAL_GAIN_THIRTY_CONSUMED_WITHOUT_DEDUCTION",
        "30% appreciated public-charity property, contribution year {0}: balance consumed in a standard-deduction year",
    ),
    (
        "LINE_CHARITY_CARRY_N_CAPITAL_GAIN_THIRTY_REMAINING",
        "30% appreciated public-charity property, contribution year {0}: closing balance",
    ),
    (
        "LINE_CHARITY_CARRY_N_CAPITAL_GAIN_THIRTY_EXPIRED",
        "30% appreciated public-charity property, contribution year {0}: expired balance",
    ),
    (
        "LINE_CHARITY_CARRY_N_CAPITAL_GAIN_THIRTY_UNUSED",
        "30% appreciated public-charity property, contribution year {0}: unused balance",
    ),
    (
        "LINE_CHARITY_CARRY_N_CAPITAL_GAIN_THIRTY_FLOOR_NOT_CARRIED",
        "30% appreciated public-charity property, contribution year {0}: floor consumed without deduction",
    ),
    (
        "LINE_CHARITY_PRIOR_CAPITAL_GAIN_THIRTY_DEDUCTION",
        "30% appreciated public-charity property: prior gifts deducted",
    ),
    (
        "LINE_CHARITY_PRIOR_CAPITAL_GAIN_THIRTY_CONSUMED_WITHOUT_DEDUCTION",
        "30% appreciated public-charity property: prior balances consumed without deduction",
    ),
    (
        "LINE_CHARITY_PRIOR_CAPITAL_GAIN_THIRTY_FLOOR_NOT_CARRIED",
        "30% appreciated public-charity property: prior balances consumed by the floor",
    ),
    (
        "LINE_CHARITY_PRIOR_CAPITAL_GAIN_THIRTY_REMAINING_N",
        "30% appreciated public-charity property: balance carried to {0}",
    ),
    (
        "LINE_CHARITY_PRIOR_CAPITAL_GAIN_THIRTY_EXPIRED_AFTER_N",
        "30% appreciated public-charity property: balance expired after {0}",
    ),
    (
        "LINE_CHARITY_CAPITAL_GAIN_TWENTY_CURRENT_DEDUCTION",
        "20% appreciated other-organization property: current gifts deducted",
    ),
    (
        "LINE_CHARITY_CAPITAL_GAIN_TWENTY_ALLOWED",
        "20% appreciated other-organization property: current and prior gifts deducted",
    ),
    (
        "LINE_CHARITY_CAPITAL_GAIN_TWENTY_FLOOR",
        "20% appreciated other-organization property: itemizer floor",
    ),
    (
        "LINE_CHARITY_CAPITAL_GAIN_TWENTY_N_CARRYFORWARD",
        "20% appreciated other-organization property: {0} excess and eligible floor carryforward",
    ),
    (
        "LINE_CHARITY_CARRY_N_CAPITAL_GAIN_TWENTY_OPENING",
        "20% appreciated other-organization property, contribution year {0}: opening balance",
    ),
    (
        "LINE_CHARITY_CARRY_N_CAPITAL_GAIN_TWENTY_USED",
        "20% appreciated other-organization property, contribution year {0}: balance used before the itemizer floor",
    ),
    (
        "LINE_CHARITY_CARRY_N_CAPITAL_GAIN_TWENTY_DEDUCTION",
        "20% appreciated other-organization property, contribution year {0}: balance deducted",
    ),
    (
        "LINE_CHARITY_CARRY_N_CAPITAL_GAIN_TWENTY_CONSUMED_WITHOUT_DEDUCTION",
        "20% appreciated other-organization property, contribution year {0}: balance consumed in a standard-deduction year",
    ),
    (
        "LINE_CHARITY_CARRY_N_CAPITAL_GAIN_TWENTY_REMAINING",
        "20% appreciated other-organization property, contribution year {0}: closing balance",
    ),
    (
        "LINE_CHARITY_CARRY_N_CAPITAL_GAIN_TWENTY_EXPIRED",
        "20% appreciated other-organization property, contribution year {0}: expired balance",
    ),
    (
        "LINE_CHARITY_CARRY_N_CAPITAL_GAIN_TWENTY_UNUSED",
        "20% appreciated other-organization property, contribution year {0}: unused balance",
    ),
    (
        "LINE_CHARITY_CARRY_N_CAPITAL_GAIN_TWENTY_FLOOR_NOT_CARRIED",
        "20% appreciated other-organization property, contribution year {0}: floor consumed without deduction",
    ),
    (
        "LINE_CHARITY_PRIOR_CAPITAL_GAIN_TWENTY_DEDUCTION",
        "20% appreciated other-organization property: prior gifts deducted",
    ),
    (
        "LINE_CHARITY_PRIOR_CAPITAL_GAIN_TWENTY_CONSUMED_WITHOUT_DEDUCTION",
        "20% appreciated other-organization property: prior balances consumed without deduction",
    ),
    (
        "LINE_CHARITY_PRIOR_CAPITAL_GAIN_TWENTY_FLOOR_NOT_CARRIED",
        "20% appreciated other-organization property: prior balances consumed by the floor",
    ),
    (
        "LINE_CHARITY_PRIOR_CAPITAL_GAIN_TWENTY_REMAINING_N",
        "20% appreciated other-organization property: balance carried to {0}",
    ),
    (
        "LINE_CHARITY_PRIOR_CAPITAL_GAIN_TWENTY_EXPIRED_AFTER_N",
        "20% appreciated other-organization property: balance expired after {0}",
    ),
    (
        "LINE_CHARITY_CONSERVATION_FIFTY_CURRENT_DEDUCTION",
        "50% conservation contributions: current gifts deducted",
    ),
    (
        "LINE_CHARITY_CONSERVATION_FIFTY_ALLOWED",
        "50% conservation contributions: current and prior gifts deducted",
    ),
    ("LINE_CHARITY_CONSERVATION_FIFTY_FLOOR", "50% conservation contributions: itemizer floor"),
    (
        "LINE_CHARITY_CONSERVATION_FIFTY_N_CARRYFORWARD",
        "50% conservation contributions: {0} excess and eligible floor carryforward",
    ),
    (
        "LINE_CHARITY_CARRY_N_CONSERVATION_FIFTY_OPENING",
        "50% conservation contributions, contribution year {0}: opening balance",
    ),
    (
        "LINE_CHARITY_CARRY_N_CONSERVATION_FIFTY_USED",
        "50% conservation contributions, contribution year {0}: balance used before the itemizer floor",
    ),
    (
        "LINE_CHARITY_CARRY_N_CONSERVATION_FIFTY_DEDUCTION",
        "50% conservation contributions, contribution year {0}: balance deducted",
    ),
    (
        "LINE_CHARITY_CARRY_N_CONSERVATION_FIFTY_CONSUMED_WITHOUT_DEDUCTION",
        "50% conservation contributions, contribution year {0}: balance consumed in a standard-deduction year",
    ),
    (
        "LINE_CHARITY_CARRY_N_CONSERVATION_FIFTY_REMAINING",
        "50% conservation contributions, contribution year {0}: closing balance",
    ),
    (
        "LINE_CHARITY_CARRY_N_CONSERVATION_FIFTY_EXPIRED",
        "50% conservation contributions, contribution year {0}: expired balance",
    ),
    (
        "LINE_CHARITY_CARRY_N_CONSERVATION_FIFTY_UNUSED",
        "50% conservation contributions, contribution year {0}: unused balance",
    ),
    (
        "LINE_CHARITY_CARRY_N_CONSERVATION_FIFTY_FLOOR_NOT_CARRIED",
        "50% conservation contributions, contribution year {0}: floor consumed without deduction",
    ),
    (
        "LINE_CHARITY_PRIOR_CONSERVATION_FIFTY_DEDUCTION",
        "50% conservation contributions: prior gifts deducted",
    ),
    (
        "LINE_CHARITY_PRIOR_CONSERVATION_FIFTY_CONSUMED_WITHOUT_DEDUCTION",
        "50% conservation contributions: prior balances consumed without deduction",
    ),
    (
        "LINE_CHARITY_PRIOR_CONSERVATION_FIFTY_FLOOR_NOT_CARRIED",
        "50% conservation contributions: prior balances consumed by the floor",
    ),
    (
        "LINE_CHARITY_PRIOR_CONSERVATION_FIFTY_REMAINING_N",
        "50% conservation contributions: balance carried to {0}",
    ),
    (
        "LINE_CHARITY_PRIOR_CONSERVATION_FIFTY_EXPIRED_AFTER_N",
        "50% conservation contributions: balance expired after {0}",
    ),
    (
        "LINE_CHARITY_CONSERVATION_HUNDRED_CURRENT_DEDUCTION",
        "100% qualified-farmer conservation contributions: current gifts deducted",
    ),
    (
        "LINE_CHARITY_CONSERVATION_HUNDRED_ALLOWED",
        "100% qualified-farmer conservation contributions: current and prior gifts deducted",
    ),
    (
        "LINE_CHARITY_CONSERVATION_HUNDRED_FLOOR",
        "100% qualified-farmer conservation contributions: itemizer floor",
    ),
    (
        "LINE_CHARITY_CONSERVATION_HUNDRED_N_CARRYFORWARD",
        "100% qualified-farmer conservation contributions: {0} excess and eligible floor carryforward",
    ),
    (
        "LINE_CHARITY_CARRY_N_CONSERVATION_HUNDRED_OPENING",
        "100% qualified-farmer conservation contributions, contribution year {0}: opening balance",
    ),
    (
        "LINE_CHARITY_CARRY_N_CONSERVATION_HUNDRED_USED",
        "100% qualified-farmer conservation contributions, contribution year {0}: balance used before the itemizer floor",
    ),
    (
        "LINE_CHARITY_CARRY_N_CONSERVATION_HUNDRED_DEDUCTION",
        "100% qualified-farmer conservation contributions, contribution year {0}: balance deducted",
    ),
    (
        "LINE_CHARITY_CARRY_N_CONSERVATION_HUNDRED_CONSUMED_WITHOUT_DEDUCTION",
        "100% qualified-farmer conservation contributions, contribution year {0}: balance consumed in a standard-deduction year",
    ),
    (
        "LINE_CHARITY_CARRY_N_CONSERVATION_HUNDRED_REMAINING",
        "100% qualified-farmer conservation contributions, contribution year {0}: closing balance",
    ),
    (
        "LINE_CHARITY_CARRY_N_CONSERVATION_HUNDRED_EXPIRED",
        "100% qualified-farmer conservation contributions, contribution year {0}: expired balance",
    ),
    (
        "LINE_CHARITY_CARRY_N_CONSERVATION_HUNDRED_UNUSED",
        "100% qualified-farmer conservation contributions, contribution year {0}: unused balance",
    ),
    (
        "LINE_CHARITY_CARRY_N_CONSERVATION_HUNDRED_FLOOR_NOT_CARRIED",
        "100% qualified-farmer conservation contributions, contribution year {0}: floor consumed without deduction",
    ),
    (
        "LINE_CHARITY_PRIOR_CONSERVATION_HUNDRED_DEDUCTION",
        "100% qualified-farmer conservation contributions: prior gifts deducted",
    ),
    (
        "LINE_CHARITY_PRIOR_CONSERVATION_HUNDRED_CONSUMED_WITHOUT_DEDUCTION",
        "100% qualified-farmer conservation contributions: prior balances consumed without deduction",
    ),
    (
        "LINE_CHARITY_PRIOR_CONSERVATION_HUNDRED_FLOOR_NOT_CARRIED",
        "100% qualified-farmer conservation contributions: prior balances consumed by the floor",
    ),
    (
        "LINE_CHARITY_PRIOR_CONSERVATION_HUNDRED_REMAINING_N",
        "100% qualified-farmer conservation contributions: balance carried to {0}",
    ),
    (
        "LINE_CHARITY_PRIOR_CONSERVATION_HUNDRED_EXPIRED_AFTER_N",
        "100% qualified-farmer conservation contributions: balance expired after {0}",
    ),
    (
        "LINE_CHARITY_DISASTER_RELIEF_CURRENT_DEDUCTION",
        "qualified 2021 disaster-relief gifts: current gifts deducted",
    ),
    (
        "LINE_CHARITY_DISASTER_RELIEF_ALLOWED",
        "qualified 2021 disaster-relief gifts: current and prior gifts deducted",
    ),
    ("LINE_CHARITY_DISASTER_RELIEF_FLOOR", "qualified 2021 disaster-relief gifts: itemizer floor"),
    (
        "LINE_CHARITY_DISASTER_RELIEF_N_CARRYFORWARD",
        "qualified 2021 disaster-relief gifts: {0} excess and eligible floor carryforward",
    ),
    (
        "LINE_CHARITY_CARRY_N_DISASTER_RELIEF_OPENING",
        "qualified 2021 disaster-relief gifts, contribution year {0}: opening balance",
    ),
    (
        "LINE_CHARITY_CARRY_N_DISASTER_RELIEF_USED",
        "qualified 2021 disaster-relief gifts, contribution year {0}: balance used before the itemizer floor",
    ),
    (
        "LINE_CHARITY_CARRY_N_DISASTER_RELIEF_DEDUCTION",
        "qualified 2021 disaster-relief gifts, contribution year {0}: balance deducted",
    ),
    (
        "LINE_CHARITY_CARRY_N_DISASTER_RELIEF_CONSUMED_WITHOUT_DEDUCTION",
        "qualified 2021 disaster-relief gifts, contribution year {0}: balance consumed in a standard-deduction year",
    ),
    (
        "LINE_CHARITY_CARRY_N_DISASTER_RELIEF_REMAINING",
        "qualified 2021 disaster-relief gifts, contribution year {0}: closing balance",
    ),
    (
        "LINE_CHARITY_CARRY_N_DISASTER_RELIEF_EXPIRED",
        "qualified 2021 disaster-relief gifts, contribution year {0}: expired balance",
    ),
    (
        "LINE_CHARITY_CARRY_N_DISASTER_RELIEF_UNUSED",
        "qualified 2021 disaster-relief gifts, contribution year {0}: unused balance",
    ),
    (
        "LINE_CHARITY_CARRY_N_DISASTER_RELIEF_FLOOR_NOT_CARRIED",
        "qualified 2021 disaster-relief gifts, contribution year {0}: floor consumed without deduction",
    ),
    (
        "LINE_CHARITY_PRIOR_DISASTER_RELIEF_DEDUCTION",
        "qualified 2021 disaster-relief gifts: prior gifts deducted",
    ),
    (
        "LINE_CHARITY_PRIOR_DISASTER_RELIEF_CONSUMED_WITHOUT_DEDUCTION",
        "qualified 2021 disaster-relief gifts: prior balances consumed without deduction",
    ),
    (
        "LINE_CHARITY_PRIOR_DISASTER_RELIEF_FLOOR_NOT_CARRIED",
        "qualified 2021 disaster-relief gifts: prior balances consumed by the floor",
    ),
    (
        "LINE_CHARITY_PRIOR_DISASTER_RELIEF_REMAINING_N",
        "qualified 2021 disaster-relief gifts: balance carried to {0}",
    ),
    (
        "LINE_CHARITY_PRIOR_DISASTER_RELIEF_EXPIRED_AFTER_N",
        "qualified 2021 disaster-relief gifts: balance expired after {0}",
    ),
    (
        "LINE_CHARITY_PROPERTY_N_CLAIM",
        "Property gift {0}: deductible claim before percentage limits",
    ),
    ("LINE_FORM_N_NEEDED", "Form {0} needed (1 = yes)"),
    ("LINE_FORM_N_SECTION_A_N_COST", "Section A gift {1}: cost or basis"),
    ("LINE_FORM_N_SECTION_A_N_VALUE", "Section A gift {1}: fair market value"),
    ("LINE_FORM_N_SECTION_A_N_CLAIMED", "Section A gift {1}: amount claimed"),
    ("LINE_FORM_N_SECTION_B_N_COST", "Section B gift {1}: cost or basis"),
    ("LINE_FORM_N_SECTION_B_N_VALUE", "Section B gift {1}: fair market value"),
    ("LINE_FORM_N_SECTION_B_N_CLAIMED", "Section B gift {1}: amount claimed"),
    (
        "EXPLAIN_CHARITY_CLASS_WORKSHEET",
        "Charitable contribution limits, oldest eligible carryovers, and the 2026 income floor are applied in the statutory class order.",
    ),
    (
        "WARN_GIFT_LEFT_OUT",
        "The contribution of {0} was left out because the selected facts do not establish a supported complete gift or the required qualified appraisal and donee signature.",
    ),
    (
        "WARN_CHARITY_CARRY_LEFT_OUT",
        "This charitable carryover was left out because it does not belong to the same filer or joint couple.",
    ),
    ("LINE_BUSINESS_CREDIT_CREDIT_SOURCE_N_FILER_N_N", "{0}, source {1}, {2}, {3} credit"),
    (
        "LINE_BUSINESS_CREDIT_CREDIT_SOURCE_N_FILER_N_N_USED",
        "{0}, source {1}, {2}, {3} credit used",
    ),
    (
        "LINE_BUSINESS_CREDIT_CREDIT_SOURCE_N_FILER_N_N_TO_N",
        "{0}, source {1}, {2}, {3} credit carried to {4}",
    ),
    (
        "LINE_BUSINESS_CREDIT_CREDIT_SOURCE_N_FILER_N_N_EXPIRED",
        "{0}, source {1}, {2}, {3} credit expired",
    ),
    (
        "LINE_BUSINESS_CREDIT_CREDIT_SOURCE_N_FILER_N_N_CARRYBACK_N",
        "{0}, source {1}, {2}, {3} credit carried back to {4}",
    ),
    (
        "LINE_BUSINESS_CREDIT_CREDIT_SOURCE_N_FILER_N_N_PASSIVE",
        "{0}, source {1}, {2}, {3} passive credit",
    ),
    (
        "LINE_BUSINESS_CREDIT_CREDIT_SOURCE_N_FILER_N_N_PASSIVE_ALLOWED",
        "{0}, source {1}, {2}, {3} passive credit allowed",
    ),
    (
        "LINE_BUSINESS_CREDIT_CREDIT_SOURCE_N_FILER_N_N_PASSIVE_TO_N",
        "{0}, source {1}, {2}, {3} passive credit carried to {4}",
    ),
    (
        "WARN_OTHER_CREDIT_LEFT_OUT",
        "A credit with no supported 2026 line is left out of this estimate; your tax may be lower.",
    ),
    (
        "WARN_ENTERED_BUSINESS_CREDIT_LEFT_OUT",
        "{0}: this credit is left out because the entered expenses or basis do not yet reflect its required reduction; your tax may be lower after correction.",
    ),
    (
        "EXPLAIN_SCHEDULE_3_ELECTIVE_PAYMENT",
        "Net elective payment after applying the credit to tax, reported on Schedule 3 line 13c.",
    ),
    (
        "EXPLAIN_FORM8582CR_ACTIVE",
        "Form 8582-CR line {0}: rental real-estate credits with active participation.",
    ),
    (
        "EXPLAIN_FORM3800_SECTION_B",
        "Form 3800 Section B, line {0}: separate empowerment-zone and renewal-community credit limit.",
    ),
    ("EXPLAIN_FORM3800_COLUMN", "Form 3800 Part {0}, line {1}, column {2}."),
    (
        "W2_SCORP_EMPLOYER_UNAVAILABLE",
        "Select the actual S-corporation K-1 whose corporation paid this W-2, or choose Not paid by one of my S corporations.",
    ),
    (
        "FARM_EXTERNAL_SOURCE_UNAVAILABLE",
        "Select an available saved income source for each farm source row.",
    ),
    (
        "DE_MINIMIS_ITEM_ABOVE_PURCHASES",
        "The largest item cannot exceed all purchases under the election.",
    ),
    (
        "EXPLAIN_OWNER_PLAN_STEP",
        "Owner plan deduction worksheet step {0,number,integer}, Publication 560.",
    ),
    (
        "EXPLAIN_OWNER_PLAN_DEDUCTION",
        "Owner retirement plan deduction, Schedule 1 line 16; it also reduces the health-insurance limit.",
    ),
    ("EXPLAIN_BUSINESS_RETURNS", "Customer returns and allowances, Schedule C line 2."),
    (
        "EXPLAIN_BUSINESS_NET_RECEIPTS",
        "Gross receipts less returns and allowances, Schedule C line 3.",
    ),
    ("EXPLAIN_BUSINESS_GROSS_PROFIT", "Net receipts less cost of goods sold, Schedule C line 5."),
    (
        "EXPLAIN_BUSINESS_INVENTORY",
        "Inventory amounts valued under the selected method, Schedule C PartIII.",
    ),
    ("EXPLAIN_BUSINESS_OTHER_INCOME", "Other business income: {0}."),
    (
        "EXPLAIN_BUSINESS_DE_MINIMIS",
        "Qualifying low-cost property deducted once under the annual de minimis safe-harbor election, Schedule C PartV.",
    ),
    (
        "EXPLAIN_LEASE_AMOUNT",
        "Business share of lease payments or passenger-automobile inclusion, Publication463.",
    ),
    (
        "EXPLAIN_BUSINESS_DEPLETION",
        "Cost or percentage depletion of this business's working interest, after the applicable income limit, Schedule C line 12.",
    ),
    (
        "EXPLAIN_BUSINESS_DEPLETION_CARRY",
        "Oil and gas percentage depletion above the taxable-income limit carried to2027.",
    ),
    (
        "EXPLAIN_BUSINESS_N_EXPENSE_LINE19_PENSION_PLANS",
        "Employee pension and profit-sharing plan contributions, Schedule C line 19; owner contributions belong on Schedule 1.",
    ),
    (
        "EXPLAIN_CARE_WORKSHEET_A",
        "Worksheet A copies the seven retained 2025 return facts and applies the 2025 credit table. Its result increases 2026 Form 2441 line 9b, subject to the 2026 credit limit.",
    ),
    ("LINE_CARE_PRIOR_WORKSHEET_A_LINE1", "2025 care Worksheet A line 1"),
    ("LINE_CARE_PRIOR_WORKSHEET_A_LINE2", "2025 care Worksheet A line 2"),
    ("LINE_CARE_PRIOR_WORKSHEET_A_LINE3", "2025 care Worksheet A line 3"),
    ("LINE_CARE_PRIOR_WORKSHEET_A_LINE4", "2025 care Worksheet A line 4"),
    ("LINE_CARE_PRIOR_WORKSHEET_A_LINE5", "2025 care Worksheet A line 5"),
    ("LINE_CARE_PRIOR_WORKSHEET_A_LINE6", "2025 care Worksheet A line 6"),
    ("LINE_CARE_PRIOR_WORKSHEET_A_LINE7", "2025 care Worksheet A line 7"),
    ("LINE_CARE_PRIOR_WORKSHEET_A_LINE8", "2025 care Worksheet A line 8"),
    ("LINE_CARE_PRIOR_WORKSHEET_A_LINE9", "2025 care Worksheet A line 9"),
    ("LINE_CARE_PRIOR_WORKSHEET_A_LINE10", "2025 care Worksheet A line 10"),
    ("LINE_CARE_PRIOR_WORKSHEET_A_LINE11", "2025 care Worksheet A line 11"),
    ("LINE_CARE_PRIOR_WORKSHEET_A_LINE12", "2025 care Worksheet A line 12"),
    (
        "EXPLAIN_CARE_SOURCE_DEDUCTION",
        "Form 2441 line 24, allocated among the selected own-business benefit sources in proportion to their received benefits. This source deduction is counted once in income and its attributable QBI; the original K-1 facts are retained.",
    ),
    ("LINE_PARTNERSHIP_N_CARE_DEDUCTION", "Partnership {0,number,0} dependent-care deduction"),
    ("LINE_CARE_TAXABLE_TAXPAYER", "Taxpayer's taxable dependent-care benefits"),
    ("LINE_CARE_TAXABLE_SPOUSE", "Spouse's taxable dependent-care benefits"),
    ("SECTION_CARE", "Child and dependent care"),
    ("CARE_SOURCE_NAME", "{0}"),
    (
        "CARE_SOURCE_AMOUNTS_UNAVAILABLE",
        "The combined dependent-care source amounts exceed the supported arithmetic range.",
    ),
    ("LINE_CARE_FORM2441_LINE12", "Form 2441 line 12"),
    ("LINE_CARE_FORM2441_LINE13", "Form 2441 line 13"),
    ("LINE_CARE_FORM2441_LINE14", "Form 2441 line 14"),
    ("LINE_CARE_FORM2441_LINE15", "Form 2441 line 15"),
    ("LINE_CARE_FORM2441_LINE16", "Form 2441 line 16"),
    ("LINE_CARE_FORM2441_LINE17", "Form 2441 line 17"),
    ("LINE_CARE_FORM2441_LINE18", "Form 2441 line 18"),
    ("LINE_CARE_FORM2441_LINE19", "Form 2441 line 19"),
    ("LINE_CARE_FORM2441_LINE20", "Form 2441 line 20"),
    ("LINE_CARE_FORM2441_LINE21", "Form 2441 line 21"),
    ("LINE_CARE_FORM2441_LINE22", "Form 2441 line 22"),
    ("LINE_CARE_FORM2441_LINE23", "Form 2441 line 23"),
    ("LINE_CARE_FORM2441_LINE24", "Form 2441 line 24"),
    ("LINE_CARE_FORM2441_LINE27", "Form 2441 line 27"),
    ("LINE_CARE_FORM2441_LINE28", "Form 2441 line 28"),
    ("LINE_CARE_FORM2441_LINE29", "Form 2441 line 29"),
    ("LINE_CARE_FORM2441_LINE30", "Form 2441 line 30"),
    ("LINE_CARE_FORM2441_LINE31", "Form 2441 line 31"),
    (
        "LINE_CARE_OWNER_TAXPAYER_USED",
        "Long-term-care distribution limit used, taxpayer (all plans)",
    ),
    ("LINE_CARE_OWNER_SPOUSE_USED", "Long-term-care distribution limit used, spouse (all plans)"),
    (
        "EXPLAIN_CARE_OWNER_USED",
        "One person's long-term-care exception is limited to $2,600 a year across every plan; the plans' rows share it in entry order.",
    ),
    ("LINE_CARE_PROVIDER_N_PAID", "Care provider {0,number,0} amount paid"),
    ("LINE_CARE_FILER_N_DEEMED", "Care filer {0,number,0} deemed earned income"),
    ("LINE_CARE_DISTRIBUTION_N_EXCEPTION", "Care distribution {0} exception"),
    ("LINE_CARE_DISTRIBUTION_N_LIMIT", "Care distribution {0} limit"),
    ("LINE_CARE_DISTRIBUTION_N_USED", "Care distribution {0} used"),
    ("LINE_CARE_PRIOR_WORKSHEET_A_LINE13", "2025 care Worksheet A line 13"),
    (
        "WARN_FOREIGN_RESIDENCE_STATEMENT",
        "The bona fide residence test is not met when you told the foreign authorities you were not resident and they did not tax you as a resident. Review whether the physical presence test applies.",
    ),
    (
        "WARN_OPTIONAL_METHOD_NOT_AVAILABLE",
        "The elected optional method does not meet its income or history tests. The estimate uses the regular method instead.",
    ),
    ("TOPIC_CHAPTER_11_BANKRUPTCY", "Chapter 11 bankruptcy"),
    (
        "EXPLAIN_CLERGY_HOUSING_EXCLUDED",
        "The furnished parsonage value and used utility allowance, or the smallest designated housing, actual housing cost and rental value, are excluded under section 107.",
    ),
    (
        "EXPLAIN_CLERGY_HOUSING_EXCESS",
        "The designated housing or utility allowance above the excluded amount is taxable as excess allowance on Form 1040 line 1h.",
    ),
    (
        "EXPLAIN_CLERGY_TAX_FREE_PERCENT",
        "Tax-free housing divided by ministerial wages, ministerial gross business income and the full housing designation determines the expense allocation.",
    ),
    (
        "EXPLAIN_CLERGY_SE_EARNINGS",
        "Ministerial wages, full housing value or designation and ministerial business profit, less allowable employee expenses, determine the ministerial ScheduleSE share before an approved exemption.",
    ),
    (
        "EXPLAIN_CLERGY_NONDEDUCTIBLE_EXPENSES",
        "The expense share allocable to tax-free housing is added back for income tax but remains deductible for self-employment tax.",
    ),
    ("LINE_CLERGY_TAXPAYER_HOUSING_EXCLUDED", "Tax-free clergy housing (you)"),
    ("LINE_CLERGY_TAXPAYER_HOUSING_EXCESS", "Taxable excess housing allowance (you)"),
    ("LINE_CLERGY_TAXPAYER_TAX_FREE_PERCENT", "Tax-free share of ministerial income (you)"),
    ("LINE_CLERGY_TAXPAYER_SE_EARNINGS", "Ministerial self-employment earnings (you)"),
    ("LINE_CLERGY_TAXPAYER_NONDEDUCTIBLE_EXPENSES", "Expenses allocable to tax-free housing (you)"),
    ("LINE_CLERGY_SPOUSE_HOUSING_EXCLUDED", "Tax-free clergy housing (your spouse)"),
    ("LINE_CLERGY_SPOUSE_HOUSING_EXCESS", "Taxable excess housing allowance (your spouse)"),
    ("LINE_CLERGY_SPOUSE_TAX_FREE_PERCENT", "Tax-free share of ministerial income (your spouse)"),
    ("LINE_CLERGY_SPOUSE_SE_EARNINGS", "Ministerial self-employment earnings (your spouse)"),
    (
        "LINE_CLERGY_SPOUSE_NONDEDUCTIBLE_EXPENSES",
        "Expenses allocable to tax-free housing (your spouse)",
    ),
    (
        "ISSUE_FARM_POSITIVE_METHOD_SOURCE",
        "Resolve the positive section 481 adjustment's AMT and consent/period facts; known receipts remain included, but required tax is unresolved.",
    ),
    (
        "NOTE_FARM_PROPERTY_DISPOSITION_SOURCE",
        "The actual property disposition remains for review. Resolve its basis and ownership history before relying on a completed tax result; known farm receipts are retained.",
    ),
    (
        "WARN_FARM_REVISED_ASSESSMENT_OMITTED",
        "The revised assessment deduction is left out until its actual recalculation is supplied. Known farm receipts remain included; disposition basis still needs its source.",
    ),
    (
        "WARN_FARM_PROPERTY_HISTORY_OMITTED",
        "Property deductions with unresolved basis or ownership history are left out. Known farm receipts remain included, and a sale still needs its actual gain/basis source.",
    ),
    (
        "WARN_FARM_AVERAGING_HISTORY_OMITTED",
        "Farm income averaging is left out because its required history is missing; known current farm income remains included.",
    ),
    (
        "FARM_METHOD_SOURCE_ID",
        "Select an actual accepted accounting-change source of this same activity and retain its approval/automatic-change evidence. Save new records, then reopen to link their assigned identities.",
    ),
    (
        "EXPLAIN_FARM_LAND_INSTALLMENT_RECAPTURE_HISTORY",
        "Retain this remaining ordinary recapture pool with the original sale and payment records. Each later payment's recognized gain takes remaining section 1252 first, then section 1255, before residual section 1231 gain. This is not immediate section 453(i) depreciation recapture.",
    ),
    (
        "LINE_FARM_N_LAND_N_COST_SHARING_RECAPTURE_REMAINING",
        "Farm {0} land {1} remaining section 1255 installment recapture",
    ),
    (
        "LINE_FARM_N_LAND_N_CONSERVATION_RECAPTURE_REMAINING",
        "Farm {0} land {1} remaining section 1252 installment recapture",
    ),
    ("FARM_PARCEL_CHOICE", "{0} — {1}"),
    (
        "EXPLAIN_FARM_LAND_RETAINED_PART",
        "The selected parcel's original opening basis and prior deductions are reduced only by their actual recorded sold-tract amounts. These balances belong to the retained land; they are not another current deduction or gain.",
    ),
    ("LINE_FARM_N_LAND_N_RETAINED_REGULAR", "Farm {0} land {1} retained regular opening basis"),
    ("LINE_FARM_N_LAND_N_RETAINED_AMT", "Farm {0} land {1} retained AMT opening basis"),
    (
        "LINE_FARM_N_LAND_N_RETAINED_CONSERVATION",
        "Farm {0} land {1} retained prior conservation deductions",
    ),
    (
        "LINE_FARM_N_LAND_N_RETAINED_AMT_CONSERVATION",
        "Farm {0} land {1} retained prior AMT conservation deductions",
    ),
    (
        "ISSUE_FARM_LAND_LINKED_BASIS_NEEDED",
        "This sale needs its accepted parcel's acquisition, regular/AMT basis and section 126 history. Unknown improvement basis or a payment after disposal cannot be omitted from required gain.",
    ),
    (
        "EXPLAIN_FARM_LAND_1255_RECAPTURE",
        "Section 1255 recapture is limited to positive gain remaining after other PartIV ordinary recapture. Its actual exclusion history uses the last certified payment date and the statutory ten-to-twenty-year percentages. Installment gain takes section 1252 first, then section 1255, as each payment's gain is recognized.",
    ),
    ("LINE_FARM_N_LAND_N_COST_SHARING_RECAPTURE", "Farm {0} land {1} section 1255 recapture"),
    (
        "EXPLAIN_FARM126_VALUE",
        "The improvement fair market value is multiplied by qualified section 126 cost divided by total improvement cost. This economic benefit differs from government cash financing.",
    ),
    (
        "EXPLAIN_FARM126_TAXABLE",
        "Qualified improvement income is its economic value minus the owner contribution and the applicable supported excludable portion, with no negative income. Separately identified nonqualified payments are added to taxable income.",
    ),
    (
        "EXPLAIN_FARM126_RECAPTURE_PORTION",
        "The section 1255 source is the regulation's applicable excludable portion after the actual partial or full no-exclusion election. It is not government cash minus taxable income and is not capped at improvement value minus owner contribution.",
    ),
    (
        "EXPLAIN_FARM126_BASIS",
        "A wholly qualified, fully funded improvement uses the supported owner contribution plus taxable improvement income. Mixed funding uses actual improvement cost less the actual excluded funding applied, separately from the income-right appraisal.",
    ),
    (
        "EXPLAIN_FARM_PARCEL_BASIS",
        "The canonical parcel opening or acquisition basis includes each supported live improvement adjustment once. Prior exclusion and conservation histories are retained recapture facts, not a second basis subtraction.",
    ),
    (
        "WARN_FARM_CONSERVATION_GROSS_SOURCE",
        "The section 175 deduction is left out because production-income classification or the required source-share treatment is unavailable. For livestock, enter its actual use in the farm asset disposal: draft, breeding, dairy or sport. Sporting livestock and custom-hire services do not count toward this limit. Known taxable income remains included; retain your costs and carry records for the completed farming-source worksheet.",
    ),
    (
        "WARN_FARM_CONSERVATION_CARRY_SOURCE",
        "The current conservation deduction uses established current costs and the farming-income limit. A historical balance for the other spouse or self-employment computation is not established. No missing carry balance or closing amount is assumed; retain the separate original worksheets.",
    ),
    ("LINE_FARM_N_COSTSHARING_N_VALUE", "Farm {0} improvement {1} qualified value"),
    ("LINE_FARM_N_COSTSHARING_N_TAXABLE", "Farm {0} improvement {1} taxable income"),
    (
        "LINE_FARM_N_COSTSHARING_N_RECAPTURE_PORTION",
        "Farm {0} improvement {1} section 1255 source portion",
    ),
    ("LINE_FARM_N_COSTSHARING_N_BASIS", "Farm {0} improvement {1} basis increase"),
    ("LINE_FARM_N_LANDBASIS_N_REGULAR", "Farm {0} parcel {1} regular basis"),
    ("LINE_FARM_N_LANDBASIS_N_AMT", "Farm {0} parcel {1} AMT basis"),
    (
        "LINE_FARM_N_ACCRUAL_GROSS_BEFORE_INVENTORY",
        "Farm {0} accrual subtotal before product inventories",
    ),
    ("LINE_FARM_N_ACCRUAL_GROSS_INCOME", "Farm {0} accrual gross income"),
    ("LINE_FARM_N_PRODUCT_COST", "Farm {0} product cost"),
    ("LINE_FARM_N_CUSTOM_HIRE_INCOME", "Farm {0} custom hire income"),
    ("LINE_FARM_N_OTHER_INCOME", "Farm {0} other income"),
    ("LINE_FARM_N_RESALE_NET", "Farm {0} net purchased-product sales"),
    ("LINE_FARM_N_WEATHER_POSTPONED", "Farm {0} weather income postponed"),
    ("LINE_FARM_N_WEATHER_PRIOR_INCLUDED", "Farm {0} prior weather income included"),
    (
        "WARN_FARM_WEATHER_SHARED_CLASS",
        "A livestock class appears on more than one farm. Its taxpayer-wide weather-election statement and attributable income must be reconciled before postponement can be calculated. Current known farm income remains included; only this postponement is omitted.",
    ),
    (
        "EXPLAIN_FARM_N_RESALE_NET",
        "Schedule F line 1c is sales of purchased products less their actual undeducted cost, including the current weather-election adjustment only once.",
    ),
    (
        "EXPLAIN_FARM_WEATHER_POSTPONED",
        "Section 451(g) class income postponed to the following year; not another current expense.",
    ),
    (
        "EXPLAIN_FARM_WEATHER_PRIOR_INCLUDED",
        "Prior-year section 451(g) income included once in this year, retaining product character.",
    ),
    (
        "ISSUE_FARM126_INCOME_BASIS_SOURCE",
        "The cost-sharing improvement needs its actual taxable income, exclusion and linked property basis calculation before return totals can be shown.",
    ),
    (
        "ISSUE_FARM_FERTILIZER_RECOVERY_SOURCE",
        "Fertilizer and lime costs need their actual election or retained capital benefit recovery before the expense and remaining basis can be shown.",
    ),
    (
        "EXPLAIN_FARM_CUSTOM_HIRE",
        "Custom-hire receipts, separately identified from production income.",
    ),
    (
        "EXPLAIN_FARM_OTHER_INCOME",
        "Other farm income, including supported signed hedge settlements and the existing method-change and fuel-tax recovery owners exactly once.",
    ),
    (
        "EXPLAIN_FARM_ACCRUAL_SUBTOTAL",
        "Accrual gross receipts before opening inventory, purchases and closing inventory.",
    ),
    (
        "EXPLAIN_FARM_PRODUCT_COST",
        "Opening inventory plus product purchases less closing inventory; a negative amount increases gross income.",
    ),
    (
        "EXPLAIN_FARM_PRODUCTION_POULTRY_CURRENT_PORTION",
        "Current part of the actual whole-month ratable period, treated as other deductible expense for the prepaid ceiling.",
    ),
    (
        "EXPLAIN_FARM_PRODUCTION_POULTRY_PREPAID_PORTION",
        "Following-year part of the actual lesser-of-12-months/useful-life period; only this part enters the shared prepaid pool.",
    ),
    (
        "LINE_FARM_N_SUPPLIES_PRODUCTION_POULTRY_CURRENT_PORTION",
        "Farm {0} current production-poultry ratable cost",
    ),
    (
        "LINE_FARM_N_SUPPLIES_PRODUCTION_POULTRY_PREPAID_PORTION",
        "Farm {0} production-poultry cost subject to the prepaid ceiling",
    ),
    (
        "EXPLAIN_FARM_PRODUCTION_RETAINED_BASIS",
        "Opening unrecovered source basis plus current required capital allocations less actual released cost. Do not repeat this cohort in ordinary expenses or inventory balances.",
    ),
    (
        "EXPLAIN_FARM_PRODUCTION_CAPITALIZED",
        "Current section 263A allocation subtracted once after showing the unreduced production expense categories. Other mandatory capital costs remain outside ordinary expenses.",
    ),
    (
        "EXPLAIN_AMT_FARM_PRODUCTION_ADJUSTMENT",
        "Regular production deductions less the separate AMT current and released cost, from the same source cohorts.",
    ),
    ("LINE_AMT_FARM_PRODUCTION_ADJUSTMENT", "AMT farm production-cost adjustment"),
    ("LINE_FARM_N_PRODUCTION_N_RETAINED_BASIS", "Farm {0} production record {1} retained basis"),
    (
        "LINE_FARM_N_PRODUCTION_N_AMT_RETAINED_BASIS",
        "Farm {0} production record {1} retained AMT basis",
    ),
    ("LINE_FARM_N_EXPENSE_LINE32F_PRODUCTION_CAPITALIZED", "Farm {0} capitalized production cost"),
    (
        "LINE_FARM_RENTAL_N_EXPENSE_LINE30G_PRODUCTION_CAPITALIZED",
        "Farm rental {0} capitalized production cost",
    ),
    ("FARM_INFORMATION_UNKNOWN", "Information-return requirement not yet established"),
    ("DISPLAY_COOP_SOURCE", "Distribution sources"),
    ("DISPLAY_COOP_CREDITS", "Issuer credits"),
    (
        "EXPLAIN_QBI_COOPERATIVE_DPAD",
        "The issuer-allocated section 199A(g) deduction is limited to taxable income before QBI minus the ordinary QBI deduction, without applying the ordinary 20% capital-gain income limit again.",
    ),
    (
        "ISSUE_COOP_ACTIVITY_NEEDED",
        "The known cooperative income needs its actual same-owner Schedule C, farm or farm-rental recipient; no activity or income is guessed.",
    ),
    (
        "ISSUE_COOP_PROPERTY_HISTORY_NEEDED",
        "The capital-purchase refund needs the canonical property owner's effective-dated regular/AMT basis or disposition result. The refund is not simply taxed as ordinary income.",
    ),
    ("LINE_AMT_DISPOSITION_ADJUSTMENT", "Property disposition adjustment for AMT"),
    (
        "EXPLAIN_AMT_DISPOSITION_ADJUSTMENT",
        "Form 6251 line 2k combines the difference between AMT and regular taxable disposition amounts after activity limits and capital netting. Those amounts already enter the AMT calculation; this row does not add them again. The QBI deduction remains the regular-tax deduction.",
    ),
    ("LINE_SECTION179_PRIOR_CARRYOVER_TOTAL", "Section 179 prior carryover remaining"),
    (
        "EXPLAIN_SECTION179_PRIOR_CARRYOVER_TOTAL",
        "Form 4562 line 10 includes prior disallowed section 179 deductions. The carryover attributable to sold K-1 property is restored to that property's basis and removed from this deduction balance; it does not reduce your outside basis again.",
    ),
    ("LINE_K1_N_SECTION179_SALE_N_PRICE", "K-1 {0}, property {1}: Gross sales price"),
    (
        "LINE_K1_N_SECTION179_SALE_N_COST",
        "K-1 {0}, property {1}: Cost or other basis plus sale expenses",
    ),
    (
        "LINE_K1_N_SECTION179_SALE_N_DEPRECIATION",
        "K-1 {0}, property {1}: Depreciation after restoring unused section 179 carryover",
    ),
    ("LINE_K1_N_SECTION179_SALE_N_BASIS", "K-1 {0}, property {1}: Adjusted basis"),
    (
        "LINE_K1_N_SECTION179_SALE_N_GAIN",
        "K-1 {0}, property {1}: Gain or loss before owner loss limits",
    ),
    (
        "LINE_K1_N_SECTION179_SALE_N_DEPRECIATION1245",
        "K-1 {0}, property {1}: Depreciation for section 1245 recapture",
    ),
    (
        "LINE_K1_N_SECTION179_SALE_N_RECAPTURE",
        "K-1 {0}, property {1}: Section 1245 ordinary recapture",
    ),
    (
        "LINE_K1_N_SECTION179_SALE_N_PARTTHREE",
        "K-1 {0}, property {1}: Section 1231 gain from Part III",
    ),
    (
        "LINE_K1_N_SECTION179_SALE_N_SECTION1231",
        "K-1 {0}, property {1}: Section 1231 gain or allowed loss to Part I",
    ),
    (
        "LINE_K1_N_SECTION179_SALE_N_ORDINARY",
        "K-1 {0}, property {1}: Ordinary sale amount after owner loss limits",
    ),
    ("LINE_AMT_K1_N_SECTION179_SALE_N_PRICE", "AMT K-1 {0}, property {1}: Gross sales price"),
    (
        "LINE_AMT_K1_N_SECTION179_SALE_N_COST",
        "AMT K-1 {0}, property {1}: Cost or other basis plus sale expenses",
    ),
    (
        "LINE_AMT_K1_N_SECTION179_SALE_N_DEPRECIATION",
        "AMT K-1 {0}, property {1}: Depreciation after restoring unused section 179 carryover",
    ),
    ("LINE_AMT_K1_N_SECTION179_SALE_N_BASIS", "AMT K-1 {0}, property {1}: Adjusted basis"),
    (
        "LINE_AMT_K1_N_SECTION179_SALE_N_GAIN",
        "AMT K-1 {0}, property {1}: Gain or loss before owner loss limits",
    ),
    (
        "LINE_AMT_K1_N_SECTION179_SALE_N_DEPRECIATION1245",
        "AMT K-1 {0}, property {1}: Depreciation for section 1245 recapture",
    ),
    (
        "LINE_AMT_K1_N_SECTION179_SALE_N_RECAPTURE",
        "AMT K-1 {0}, property {1}: Section 1245 ordinary recapture",
    ),
    (
        "LINE_AMT_K1_N_SECTION179_SALE_N_PARTTHREE",
        "AMT K-1 {0}, property {1}: Section 1231 gain from Part III",
    ),
    (
        "LINE_AMT_K1_N_SECTION179_SALE_N_SECTION1231",
        "AMT K-1 {0}, property {1}: Section 1231 gain or allowed loss to Part I",
    ),
    (
        "LINE_AMT_K1_N_SECTION179_SALE_N_ORDINARY",
        "AMT K-1 {0}, property {1}: Ordinary sale amount after owner loss limits",
    ),
    ("LINE_FORM4797_LINE14", "Form 4797: Ordinary depreciation recapture"),
    ("LINE_FORM4797_LINE33", "Form 4797: Total depreciation recapture"),
    ("LINE_FORM4797_LINE19", "Form 4797: Total ordinary gain or loss"),
    ("LINE_FORM4797_LINE20B", "Form 4797: Ordinary gain or loss to Schedule 1"),
    (
        "EXPLAIN_K1_SECTION179_SALE",
        "The partners and shareholders worksheet restores the unused section 179 carryover to adjusted basis before finding the gain or loss. Sale losses shown here reflect basis, at-risk and passive limits; section 1231 items also enter the shared return-wide worksheet.",
    ),
    (
        "EXPLAIN_FORM4797_RECAPTURE_TOTAL",
        "Form 4797 Part III ordinary recapture transfers through line 33 to Part II line 14. It is included once in Schedule 1 line 4.",
    ),
    (
        "EXPLAIN_FORM4797_ORDINARY_TOTAL",
        "Form 4797 Part II ordinary gain or loss transfers to Schedule 1 line 4. Business-property losses keep their applicable owner and return-wide limitations.",
    ),
    (
        "RENTAL_MORTGAGE_SINGLE_SOURCE",
        "For personal-use property, save the whole-property ownership share on Mortgage and select its original loans in Rentals. Leave Rental mortgage interest zero; do not enter those points or mortgage insurance again. Links require personal-use days.",
    ),
    (
        "RENTAL_MORTGAGE_ROOT_TWICE",
        "A saved mortgage loan can belong to only one whole-property rental. Split-property loan allocations are not supported; correct the loan selections in Rentals.",
    ),
    (
        "EXPLAIN_SCHEDULE_E_TOTALS",
        "Schedule E Part I totals reconcile the property columns and allowed rental losses; royalty losses are included on line 25 separately.",
    ),
    ("LINE_SCHEDULE_E_LINE23A_RENTS", "Schedule E line 23a rents"),
    ("LINE_SCHEDULE_E_LINE23C_MORTGAGE_INTEREST", "Schedule E line 23c mortgage interest"),
    (
        "LINE_SCHEDULE_E_LINE23D_DEPRECIATION_DEPLETION",
        "Schedule E line 23d depreciation and depletion",
    ),
    ("LINE_SCHEDULE_E_LINE23E_EXPENSES", "Schedule E line 23e expenses"),
    ("LINE_SCHEDULE_E_LINE34A_PASSIVE_INCOME", "Schedule E line 34a passive income"),
    ("LINE_SCHEDULE_E_LINE34A_OTHER_INCOME", "Schedule E line 34a other income"),
    ("LINE_SCHEDULE_E_LINE34B_PASSIVE_LOSSES", "Schedule E line 34b passive deductions and losses"),
    ("LINE_SCHEDULE_E_LINE34B_OTHER_LOSSES", "Schedule E line 34b other deductions and losses"),
    ("LINE_SCHEDULE_E_LINE35_ESTATE_INCOME", "Schedule E line 35 estate and trust income"),
    ("LINE_SCHEDULE_E_LINE36_ESTATE_LOSSES", "Schedule E line 36 estate and trust losses"),
    ("LINE_SCHEDULE_E_LINE37_ESTATE_NET", "Schedule E line 37 estate and trust net income or loss"),
    ("LINE_SCHEDULE_E_LINE24_INCOME", "Schedule E line 24 income"),
    ("LINE_SCHEDULE_E_LINE25_RENTAL_LOSSES", "Schedule E line 25 rental losses"),
    ("LINE_RENTAL_N_CASUALTY_N_LOSS_BASIS", "Rental {0} property {1} casualty loss basis"),
    ("LINE_RENTAL_N_CASUALTY_N_GAIN_BASIS", "Rental {0} property {1} casualty gain basis"),
    (
        "LINE_RENTAL_N_CASUALTY_N_REALIZED_GAIN",
        "Rental {0} property {1} casualty realized gain from its gain basis",
    ),
    ("LINE_AMT_RENTAL_N_CASUALTY_N_LOSS_BASIS", "AMT Rental {0} property {1} casualty loss basis"),
    ("LINE_AMT_RENTAL_N_CASUALTY_N_GAIN_BASIS", "AMT Rental {0} property {1} casualty gain basis"),
    (
        "LINE_AMT_RENTAL_N_CASUALTY_N_REALIZED_GAIN",
        "AMT Rental {0} property {1} casualty realized gain from its gain basis",
    ),
    ("LINE_RENTAL_N_CASUALTY_N_LINE20", "Rental {0} property {1} casualty adjusted loss basis"),
    (
        "LINE_RENTAL_N_CASUALTY_N_LINE21",
        "Rental {0} property {1} casualty reimbursement received or expected",
    ),
    ("LINE_RENTAL_N_CASUALTY_N_LINE22", "Rental {0} property {1} casualty realized gain"),
    ("LINE_RENTAL_N_CASUALTY_N_LINE23", "Rental {0} property {1} casualty value before the event"),
    ("LINE_RENTAL_N_CASUALTY_N_LINE24", "Rental {0} property {1} casualty value after the event"),
    ("LINE_RENTAL_N_CASUALTY_N_LINE25", "Rental {0} property {1} casualty decrease in value"),
    ("LINE_RENTAL_N_CASUALTY_N_LINE26", "Rental {0} property {1} casualty basis or value limit"),
    (
        "LINE_RENTAL_N_CASUALTY_N_ORDINARY",
        "Rental {0} property {1} ordinary casualty loss after activity limits",
    ),
    (
        "EXPLAIN_RENTAL_CASUALTY_ACTIVITY",
        "The recurring casualty's Section B loss passes through this rental's existing at-risk and passive-activity limits. It is reported separately from Schedule E expenses.",
    ),
    (
        "LINE_RENTAL_N_CASUALTY_N_LINE27",
        "Rental {0} property {1} casualty loss before activity limits",
    ),
    (
        "LINE_AMT_RENTAL_N_CASUALTY_N_LINE27",
        "AMT Rental {0} property {1} casualty loss before activity limits",
    ),
    (
        "LINE_RENTAL_N_CASUALTY_N_RAW_LOSS",
        "Rental {0} property {1} casualty raw loss before deduction limits",
    ),
    (
        "LINE_RENTAL_N_CASUALTY_N_CONTINUING_BASIS",
        "Rental {0} property {1} casualty basis after reimbursement and loss",
    ),
    (
        "LINE_AMT_RENTAL_N_CASUALTY_N_LINE20",
        "AMT Rental {0} property {1} casualty adjusted loss basis",
    ),
    (
        "LINE_AMT_RENTAL_N_CASUALTY_N_LINE21",
        "AMT Rental {0} property {1} casualty reimbursement received or expected",
    ),
    ("LINE_AMT_RENTAL_N_CASUALTY_N_LINE22", "AMT Rental {0} property {1} casualty realized gain"),
    (
        "LINE_AMT_RENTAL_N_CASUALTY_N_LINE23",
        "AMT Rental {0} property {1} casualty value before the event",
    ),
    (
        "LINE_AMT_RENTAL_N_CASUALTY_N_LINE24",
        "AMT Rental {0} property {1} casualty value after the event",
    ),
    (
        "LINE_AMT_RENTAL_N_CASUALTY_N_LINE25",
        "AMT Rental {0} property {1} casualty decrease in value",
    ),
    (
        "LINE_AMT_RENTAL_N_CASUALTY_N_LINE26",
        "AMT Rental {0} property {1} casualty basis or value limit",
    ),
    (
        "LINE_AMT_RENTAL_N_CASUALTY_N_RAW_LOSS",
        "AMT Rental {0} property {1} casualty raw loss before deduction limits",
    ),
    (
        "LINE_AMT_RENTAL_N_CASUALTY_N_CONTINUING_BASIS",
        "AMT Rental {0} property {1} casualty basis after reimbursement and loss",
    ),
    (
        "EXPLAIN_RENTAL_CASUALTY_SOURCE",
        "Event-date property facts and any shown line 27 loss precede activity limits. Remaining basis does not establish a completed continuing-depreciation schedule.",
    ),
    (
        "RENTAL_CASUALTY_DEDUCTION_CONTEXT_UNAVAILABLE",
        "This home casualty needs the return's actual deduction choice before its allowed loss can be calculated.",
    ),
    (
        "RENTAL_CASUALTY_PERSONAL_ALLOWANCE_UNAVAILABLE",
        "The personal casualty records need their shared allowance worksheet before this rental's casualty loss is available. Resolve any personal casualty gains, qualified-disaster loss or property allocation on Other itemized deductions.",
    ),
    (
        "RENTAL_CASUALTY_HOME_ALLOWANCE_UNAVAILABLE",
        "This home casualty needs its shared personal-casualty allowance worksheet, including the actual deduction choice, qualified-disaster increase and any personal casualty gains. Its source loss and basis records remain available.",
    ),
    ("LINE_RENTAL_N_ASSET_N_MACRS", "Rental {0} asset {1} macrs depreciation"),
    ("LINE_RENTAL_N_ASSET_N_BONUS", "Rental {0} asset {1} bonus depreciation"),
    ("LINE_RENTAL_N_AT_RISK_ORDINARY_CARRYOVER", "Rental {0} ordinary at-risk loss carryover"),
    (
        "LINE_RENTAL_N_AT_RISK_SECTION1231_CARRYOVER",
        "Rental {0} section 1231 at-risk loss carryover",
    ),
    ("LINE_RENTAL_N_AT_RISK_ALLOWED_ORDINARY", "Rental {0} ordinary amount after at-risk limit"),
    (
        "LINE_RENTAL_N_AT_RISK_ALLOWED_SECTION1231",
        "Rental {0} section 1231 amount after at-risk limit",
    ),
    ("LINE_RENTAL_N_SALE_BUILDING", "Rental {0} building gain or loss"),
    ("LINE_RENTAL_N_SALE_AT_RISK_ADJUSTMENT", "Rental {0} property loss after at-risk allocation"),
    ("LINE_RENTAL_N_AT_RISK_CARRYOVER", "Rental {0} at-risk loss carried to 2027"),
    ("LINE_AMT_RENTAL_N_AT_RISK_CARRYOVER", "AMT rental {0} at-risk loss carried to 2027"),
    (
        "LINE_AMT_RENTAL_N_AT_RISK_ORDINARY_CARRYOVER",
        "AMT rental {0} ordinary at-risk loss carryover",
    ),
    (
        "LINE_AMT_RENTAL_N_AT_RISK_SECTION1231_CARRYOVER",
        "AMT rental {0} section 1231 at-risk loss carryover",
    ),
    (
        "LINE_AMT_RENTAL_N_AT_RISK_ALLOWED_ORDINARY",
        "AMT rental {0} ordinary amount after at-risk limit",
    ),
    (
        "LINE_AMT_RENTAL_N_AT_RISK_ALLOWED_SECTION1231",
        "AMT rental {0} section 1231 amount after at-risk limit",
    ),
    ("LINE_RENTAL_N_ASSET_N_DEPRECIATION", "Rental {0} asset {1} depreciation"),
    (
        "LINE_SCHEDULE_E_LINE43_PROFESSIONAL_RENTALS",
        "Schedule E line 43 professional rental income or loss",
    ),
    ("LINE_SCHEDULE_E_LINE23B_ROYALTIES", "Schedule E line 23b royalties received"),
    ("LINE_SCHEDULE_E_LINE25_ROYALTY_LOSSES", "Schedule E line 25 royalty losses"),
    ("LINE_FORM6251_LINE2D_DEPLETION", "Form 6251 line 2d depletion adjustment"),
    ("LINE_FORM6251_LINE2M_K1_PASSIVE", "Form 6251 line 2m K-1 passive activity adjustment"),
    ("LINE_FORM6251_LINE2N_K1_LOSS_LIMITS", "Form 6251 line 2n K-1 loss limitation adjustment"),
    ("LINE_BUSINESS_N_AT_RISK_ORDINARY_CARRYOVER", "Business {0} ordinary at-risk loss carryover"),
    (
        "LINE_BUSINESS_N_AT_RISK_SECTION1231_CARRYOVER",
        "Business {0} section 1231 at-risk loss carryover",
    ),
    (
        "LINE_BUSINESS_N_AT_RISK_ALLOWED_ORDINARY",
        "Business {0} ordinary amount after at-risk limit",
    ),
    (
        "LINE_BUSINESS_N_AT_RISK_ALLOWED_SECTION1231",
        "Business {0} section 1231 amount after at-risk limit",
    ),
    (
        "LINE_BUSINESS_N_SALE_AT_RISK_ADJUSTMENT",
        "Business {0} property loss after at-risk allocation",
    ),
    ("LINE_BUSINESS_N_AT_RISK_CARRYOVER", "Business {0} at-risk loss carried to 2027"),
    ("LINE_AMT_BUSINESS_N_AT_RISK_CARRYOVER", "AMT business {0} at-risk loss carried to 2027"),
    (
        "LINE_AMT_BUSINESS_N_AT_RISK_ORDINARY_CARRYOVER",
        "AMT business {0} ordinary at-risk loss carryover",
    ),
    (
        "LINE_AMT_BUSINESS_N_AT_RISK_SECTION1231_CARRYOVER",
        "AMT business {0} section 1231 at-risk loss carryover",
    ),
    (
        "LINE_AMT_BUSINESS_N_AT_RISK_ALLOWED_ORDINARY",
        "AMT business {0} ordinary amount after at-risk limit",
    ),
    (
        "LINE_AMT_BUSINESS_N_AT_RISK_ALLOWED_SECTION1231",
        "AMT business {0} section 1231 amount after at-risk limit",
    ),
    ("LINE_BUSINESS_N_FORM6198_LINE1", "Business {0} Form 6198 line 1"),
    ("LINE_BUSINESS_N_FORM6198_LINE2A", "Business {0} Form 6198 line 2a"),
    ("LINE_BUSINESS_N_FORM6198_LINE2B", "Business {0} Form 6198 line 2b"),
    ("LINE_BUSINESS_N_FORM6198_LINE2C", "Business {0} Form 6198 line 2c"),
    ("LINE_BUSINESS_N_FORM6198_LINE3", "Business {0} Form 6198 line 3"),
    ("LINE_BUSINESS_N_FORM6198_LINE4", "Business {0} Form 6198 line 4"),
    ("LINE_BUSINESS_N_FORM6198_LINE5", "Business {0} Form 6198 line 5"),
    ("LINE_BUSINESS_N_FORM6198_LINE6", "Business {0} Form 6198 line 6"),
    ("LINE_BUSINESS_N_FORM6198_LINE7", "Business {0} Form 6198 line 7"),
    ("LINE_BUSINESS_N_FORM6198_LINE8", "Business {0} Form 6198 line 8"),
    ("LINE_BUSINESS_N_FORM6198_LINE9", "Business {0} Form 6198 line 9"),
    ("LINE_BUSINESS_N_FORM6198_LINE10A", "Business {0} Form 6198 line 10a"),
    ("LINE_BUSINESS_N_FORM6198_LINE10B", "Business {0} Form 6198 line 10b"),
    ("LINE_BUSINESS_N_FORM6198_LINE20", "Business {0} Form 6198 line 20"),
    ("LINE_BUSINESS_N_FORM6198_LINE21", "Business {0} Form 6198 line 21"),
    (
        "LINE_AMT_BUSINESS_N_AT_RISK_ALLOWED_ORDINARY_PROPERTY",
        "AMT business {0} at-risk allowed ordinary property",
    ),
    (
        "LINE_AMT_BUSINESS_N_AT_RISK_ORDINARY_PROPERTY_CARRYOVER",
        "AMT business {0} at-risk ordinary property carryover",
    ),
    (
        "LINE_BUSINESS_N_AT_RISK_ALLOWED_ORDINARY_PROPERTY",
        "Business {0} at-risk allowed ordinary property",
    ),
    (
        "LINE_BUSINESS_N_AT_RISK_ORDINARY_PROPERTY_CARRYOVER",
        "Business {0} at-risk ordinary property carryover",
    ),
    ("LINE_AMT_BUSINESS_N_AT_RISK_RECAPTURE", "AMT business {0} at-risk loss recapture income"),
    (
        "LINE_AMT_BUSINESS_N_AT_RISK_RECAPTURE_DEDUCTION_NEXT_YEAR",
        "AMT business {0} recapture deduction for 2027",
    ),
    ("LINE_BUSINESS_N_AT_RISK_RECAPTURE", "Business {0} at-risk loss recapture income"),
    (
        "LINE_BUSINESS_N_AT_RISK_RECAPTURE_DEDUCTION_NEXT_YEAR",
        "Business {0} recapture deduction for 2027",
    ),
    ("LINE_BUSINESS_N_FORM6198_LINE11", "Business {0} Form 6198 line 11"),
    ("LINE_BUSINESS_N_FORM6198_LINE12", "Business {0} Form 6198 line 12"),
    ("LINE_BUSINESS_N_FORM6198_LINE13", "Business {0} Form 6198 line 13"),
    ("LINE_BUSINESS_N_FORM6198_LINE14", "Business {0} Form 6198 line 14"),
    ("LINE_BUSINESS_N_FORM6198_LINE15A", "Business {0} Form 6198 line 15a"),
    ("LINE_BUSINESS_N_FORM6198_LINE15B", "Business {0} Form 6198 line 15b"),
    ("LINE_BUSINESS_N_FORM6198_LINE16", "Business {0} Form 6198 line 16"),
    ("LINE_BUSINESS_N_FORM6198_LINE17", "Business {0} Form 6198 line 17"),
    ("LINE_BUSINESS_N_FORM6198_LINE18", "Business {0} Form 6198 line 18"),
    ("LINE_BUSINESS_N_FORM6198_LINE19A", "Business {0} Form 6198 line 19a"),
    ("LINE_BUSINESS_N_FORM6198_LINE19B", "Business {0} Form 6198 line 19b"),
    ("LINE_BUSINESS_N_PASSIVE_CURRENT", "Business {0} current passive operating result"),
    ("LINE_AMT_BUSINESS_N_PASSIVE_CURRENT", "AMT business {0} current passive operating result"),
    ("LINE_BUSINESS_N_PASSIVE_ALLOWED", "Business {0} allowed passive operating result"),
    ("LINE_AMT_BUSINESS_N_PASSIVE_ALLOWED", "AMT business {0} allowed passive operating result"),
    ("LINE_BUSINESS_N_PASSIVE_SUSPENDED", "Business {0} passive operating loss carried forward"),
    (
        "LINE_AMT_BUSINESS_N_PASSIVE_SUSPENDED",
        "AMT business {0} passive operating loss carried forward",
    ),
    (
        "EXPLAIN_BUSINESS_PASSIVE_CURRENT",
        "Current operating result after selected deductions and any at-risk limit, before the common passive loss limit.",
    ),
    (
        "EXPLAIN_BUSINESS_PASSIVE_ALLOWED",
        "This business's operating profit or allowed current loss from the same Form 8582 passive pool. It remains Schedule C income and retains its self-employment character.",
    ),
    (
        "EXPLAIN_BUSINESS_PASSIVE_SUSPENDED",
        "Current operating loss held by the passive activity limit; keep this source's regular and AMT carryovers separately.",
    ),
    (
        "BUSINESS_RISK_PROPERTY_SOURCE_REQUIRED",
        "Resolve this business property's loss character and at-risk allocation before completing its calculation.",
    ),
    ("LINE_RENTAL_N_FORM6198_LINE1", "Rental {0} Form 6198 line 1"),
    ("LINE_RENTAL_N_FORM6198_LINE2A", "Rental {0} Form 6198 line 2a"),
    ("LINE_RENTAL_N_FORM6198_LINE2B", "Rental {0} Form 6198 line 2b"),
    ("LINE_RENTAL_N_FORM6198_LINE2C", "Rental {0} Form 6198 line 2c"),
    ("LINE_RENTAL_N_FORM6198_LINE3", "Rental {0} Form 6198 line 3"),
    ("LINE_RENTAL_N_FORM6198_LINE4", "Rental {0} Form 6198 line 4"),
    ("LINE_RENTAL_N_FORM6198_LINE5", "Rental {0} Form 6198 line 5"),
    ("LINE_RENTAL_N_FORM6198_LINE6", "Rental {0} Form 6198 line 6"),
    ("LINE_RENTAL_N_FORM6198_LINE7", "Rental {0} Form 6198 line 7"),
    ("LINE_RENTAL_N_FORM6198_LINE8", "Rental {0} Form 6198 line 8"),
    ("LINE_RENTAL_N_FORM6198_LINE9", "Rental {0} Form 6198 line 9"),
    ("LINE_RENTAL_N_FORM6198_LINE10A", "Rental {0} Form 6198 line 10a"),
    ("LINE_RENTAL_N_FORM6198_LINE10B", "Rental {0} Form 6198 line 10b"),
    ("LINE_RENTAL_N_FORM6198_LINE20", "Rental {0} Form 6198 line 20"),
    ("LINE_RENTAL_N_FORM6198_LINE21", "Rental {0} Form 6198 line 21"),
    ("LINE_ROYALTY_N_LINE18_DEPLETION", "Royalty {0} line 18 depletion"),
    ("LINE_ROYALTY_N_LINE20_TOTAL_EXPENSES", "Royalty {0} line 20 total expenses"),
    ("LINE_ROYALTY_N_LINE21_INCOME", "Royalty {0} line 21 income"),
    ("LINE_ROYALTY_N_DEPLETION_CARRYOVER", "Royalty {0} depletion carryover"),
    ("LINE_AMT_ROYALTY_N_DEPLETION_CARRYOVER", "AMT royalty {0} depletion carryover"),
    ("LINE_ROYALTY_N_BASIS_END", "Royalty {0} basis end"),
    ("LINE_AMT_ROYALTY_N_BASIS_END", "AMT royalty {0} basis end"),
    ("LINE_ROYALTY_N_EXPENSE_LINE5_ADVERTISING", "Royalty {0} line 5 advertising"),
    ("LINE_ROYALTY_N_EXPENSE_LINE6_AUTO_AND_TRAVEL", "Royalty {0} line 6 auto and travel"),
    (
        "LINE_ROYALTY_N_EXPENSE_LINE7_CLEANING_AND_MAINTENANCE",
        "Royalty {0} line 7 cleaning and maintenance",
    ),
    ("LINE_ROYALTY_N_EXPENSE_LINE8_COMMISSIONS", "Royalty {0} line 8 commissions"),
    ("LINE_ROYALTY_N_EXPENSE_LINE9_INSURANCE", "Royalty {0} line 9 insurance"),
    (
        "LINE_ROYALTY_N_EXPENSE_LINE10_LEGAL_AND_PROFESSIONAL",
        "Royalty {0} line 10 legal and professional",
    ),
    ("LINE_ROYALTY_N_EXPENSE_LINE11_MANAGEMENT_FEES", "Royalty {0} line 11 management fees"),
    ("LINE_ROYALTY_N_EXPENSE_LINE12_MORTGAGE_INTEREST", "Royalty {0} line 12 mortgage interest"),
    (
        "LINE_ROYALTY_N_EXPENSE_LINE13A_VEHICLE_LOAN_INTEREST",
        "Royalty {0} line 13a vehicle loan interest",
    ),
    ("LINE_ROYALTY_N_EXPENSE_LINE13B_OTHER_INTEREST", "Royalty {0} line 13b other interest"),
    ("LINE_ROYALTY_N_EXPENSE_LINE14_REPAIRS", "Royalty {0} line 14 repairs"),
    ("LINE_ROYALTY_N_EXPENSE_LINE15_SUPPLIES", "Royalty {0} line 15 supplies"),
    ("LINE_ROYALTY_N_EXPENSE_LINE16_TAXES", "Royalty {0} line 16 taxes and licenses"),
    ("LINE_ROYALTY_N_EXPENSE_LINE17_UTILITIES", "Royalty {0} line 17 utilities"),
    ("LINE_ROYALTY_N_EXPENSE_LINE19_OTHER", "Royalty {0} line 19 other"),
    (
        "EXPLAIN_ROYALTY_COLUMN",
        "Schedule E royalty receipts less the expense lines and allowable cost or percentage depletion.",
    ),
    (
        "EXPLAIN_ROYALTY_RECORDS",
        "Retain this separately calculated basis or depletion carryover with your next-year royalty records.",
    ),
    (
        "EXPLAIN_ROYALTY_AMT_DEPLETION",
        "Regular depletion less AMT depletion; oil and gas owners have no basis preference, and the 65% limit is figured separately.",
    ),
    (
        "EXPLAIN_RENTAL_AT_RISK",
        "Form 6198 limits the loss before the passive-activity rules; the excess carries to next year.",
    ),
    (
        "EXPLAIN_RENTAL_ASSET_DEPRECIATION",
        "Rental asset depreciation uses the return-wide MACRS convention and the asset's own elections and history.",
    ),
    (
        "EXPLAIN_SCHEDULE_E_PROFESSIONAL_RENTALS",
        "Income or loss reported from rental real estate activities in which a real estate professional materially participated.",
    ),
    (
        "EXPLAIN_RENTAL_AT_RISK_RECAPTURE",
        "Recapture is income of this activity after its current at-risk limit and becomes a separate deduction next year; it does not release the current at-risk loss.",
    ),
    (
        "RISK_RECAPTURE_DETAILED_REQUIRED",
        "Supply the detailed at-risk balance before calculating recapture from a negative simplified worksheet.",
    ),
    (
        "LINE_AMT_RENTAL_N_AT_RISK_ALLOWED_ORDINARY_PROPERTY",
        "AMT rental {0} at-risk allowed ordinary property",
    ),
    (
        "LINE_AMT_RENTAL_N_AT_RISK_ORDINARY_PROPERTY_CARRYOVER",
        "AMT rental {0} at-risk ordinary property carryover",
    ),
    (
        "LINE_RENTAL_N_AT_RISK_ALLOWED_ORDINARY_PROPERTY",
        "Rental {0} at-risk allowed ordinary property",
    ),
    (
        "LINE_RENTAL_N_AT_RISK_ORDINARY_PROPERTY_CARRYOVER",
        "Rental {0} at-risk ordinary property carryover",
    ),
    ("LINE_AMT_RENTAL_N_AT_RISK_RECAPTURE", "AMT rental {0} at-risk loss recapture income"),
    (
        "LINE_AMT_RENTAL_N_AT_RISK_RECAPTURE_DEDUCTION_NEXT_YEAR",
        "AMT rental {0} recapture deduction for 2027",
    ),
    ("LINE_RENTAL_N_AT_RISK_RECAPTURE", "Rental {0} at-risk loss recapture income"),
    (
        "LINE_RENTAL_N_AT_RISK_RECAPTURE_DEDUCTION_NEXT_YEAR",
        "Rental {0} recapture deduction for 2027",
    ),
    ("LINE_RENTAL_N_FORM6198_LINE11", "Rental {0} Form 6198 line 11"),
    ("LINE_RENTAL_N_FORM6198_LINE12", "Rental {0} Form 6198 line 12"),
    ("LINE_RENTAL_N_FORM6198_LINE13", "Rental {0} Form 6198 line 13"),
    ("LINE_RENTAL_N_FORM6198_LINE14", "Rental {0} Form 6198 line 14"),
    ("LINE_RENTAL_N_FORM6198_LINE15A", "Rental {0} Form 6198 line 15a"),
    ("LINE_RENTAL_N_FORM6198_LINE15B", "Rental {0} Form 6198 line 15b"),
    ("LINE_RENTAL_N_FORM6198_LINE16", "Rental {0} Form 6198 line 16"),
    ("LINE_RENTAL_N_FORM6198_LINE17", "Rental {0} Form 6198 line 17"),
    ("LINE_RENTAL_N_FORM6198_LINE18", "Rental {0} Form 6198 line 18"),
    ("LINE_RENTAL_N_FORM6198_LINE19A", "Rental {0} Form 6198 line 19a"),
    ("LINE_RENTAL_N_FORM6198_LINE19B", "Rental {0} Form 6198 line 19b"),
    (
        "ACTIVITY_LOSS_AMOUNT_UNREPRESENTABLE",
        "The activity's combined loss or at-risk amounts cannot be represented; review the actual source amounts before calculating this return.",
    ),
    ("RISK_METHOD_INVALID", "Complete valid regular and AMT risk-method facts for this activity."),
    (
        "ACTIVITY_GROUP_INDIVIDUAL_RISK_REQUIRED",
        "Supply this activity's individual at-risk facts on its income page.",
    ),
    (
        "ACTIVITY_LOSS_HISTORY_INVALID",
        "Review this source loss's limiting stage, original year, tax character, regular and AMT amounts, and qualified portions.",
    ),
    (
        "SHORT_TERM_PARTICIPATION_AGREES",
        "Use the same material-participation answer for this short-stay business and its QBI treatment.",
    ),
    ("TOPIC_REMIC_RESIDUAL", "REMIC residual interest"),
    (
        "WARN_HOME_RENTAL_DEDUCTION_METHOD",
        "The home-use worksheet was recalculated using the standard deduction. Review the deduction method and home rental carryovers.",
    ),
    (
        "WARN_TAX_CREDIT_K1_INTERFACE_LEFT_OUT",
        "The K-1 owner's accepted credit interface is not available for this pass-through credit. Only that credit is left out; independently entered designated income is retained.",
    ),
    (
        "DOCUMENT_SAVINGS_BONDS_RECORDS",
        "Enter the supporting bond redemption, issue date, accepted interest source and education records for this Form 8815.",
    ),
    (
        "SAVINGS_BONDS_SOURCE_INVALID",
        "Select the accepted interest statement or annual instrument for these bonds, with the same owner and enough retained redemption interest. A linked source cannot be removed while this record remains.",
    ),
    ("BOND_PREMIUM_INSTRUMENT_LABEL", "{0}: {1}"),
    (
        "ISSUE_DEBT_CONTINGENT_INCOME",
        "This contingent-payment instrument needs its comparable-yield and projected-payment adjustments. Its required annual income is unresolved; complete that computation before relying on the estimate.",
    ),
    (
        "COVERDELL_INVALID_SOURCE",
        "Enter valid Coverdell beneficiary, contribution, correction and retained Form 5329 source amounts and dates.",
    ),
    (
        "EXPLAIN_COVERDELL_CORRECTION_EARNINGS",
        "Positive net earnings on a timely returned Coverdell contribution are taxable in the original contribution year, without the additional early-distribution tax. The linked Form 1099-Q owns the same earnings when supplied; a loss is not deducted.",
    ),
    (
        "EXPLAIN_COVERDELL_PART_V",
        "Form 5329 Part V aggregates this beneficiary’s accounts, donor limits, timely corrections, retained excess and non-rollover distributions. The 6% tax is limited by the combined year-end account value.",
    ),
    ("LINE_COVERDELL_TAXPAYER_LINE_N", "Coverdell taxpayer Form 5329 line {0}"),
    ("LINE_COVERDELL_SPOUSE_LINE_N", "Coverdell spouse Form 5329 line {0}"),
    ("LINE_COVERDELL_CORRECTION_N_EARNINGS", "Coverdell correction {0} taxable earnings"),
    (
        "WARN_QOF_INVESTMENT_LEFT_OUT",
        "{0}: this opportunity fund investment needs a treatment that isn't supported, so its deferred gain is left out of this estimate; your tax may be higher.",
    ),
    (
        "WARN_QOF_DEFERRAL_NOT_ALLOWED",
        "{0}: an investment made more than 180 days after a sale can't defer its gain, so no gain is deferred.",
    ),
    (
        "WARN_QOF_DEFERRAL_LEFT_OUT",
        "{0}: this deferral needs a treatment that isn't supported, so no gain is deferred; your tax may be lower.",
    ),
    (
        "SALE_TERM_NEEDED",
        "Choose short-term or long-term; the form doesn't show it, so use your purchase records.",
    ),
    (
        "SALE_NEGATIVE_PROCEEDS_WRITTEN_OPTION_ONLY",
        "Only an option you wrote can have negative proceeds.",
    ),
    (
        "SALE_WRITTEN_OPTION_SHORT_TERM",
        "An option you wrote is short-term however long it was open.",
    ),
    (
        "SALE_OPTION_PREMIUM_ON_STOCK",
        "An option premium adjustment belongs on the sale of the shares, an investment.",
    ),
    ("SALE_MARKET_DISCOUNT_NEEDS_BASIS", "Enter the bond's basis to figure the market discount."),
    ("SALE_MARKET_DISCOUNT_ON_A_BOND", "Market discount applies to a bond held as an investment."),
    ("DIV_SECTION_1202_ABOVE_2A", "Box 2c cannot exceed box 2a total capital gain distributions."),
    ("DIV_SECTION_1202_NEEDS_EXCLUSION", "Choose the section 1202 exclusion for the box 2c gain."),
    (
        "EXPLAIN_FORM_8949_ROW",
        "{0}: Form 8949 box {1}, column (f) code {2}; columns (d), (e), (g) and (h) add to the Schedule D line for this box.",
    ),
    (
        "EXPLAIN_FORM_8949_ROW_NO_CODES",
        "{0}: Form 8949 box {1}; columns (d), (e), (g) and (h) add to the Schedule D line for this box.",
    ),
    (
        "EXPLAIN_SALE_N_MARKET_DISCOUNT",
        "Accrued market discount recognized on the sale of a bond (Form 8949 code D): the smaller of the gain and the accrued discount is interest on Schedule B line 1, not capital gain.",
    ),
    (
        "EXPLAIN_QOF_N_BASIS_INCREASE",
        "Basis added to the opportunity fund investment for the years you held it: 10% of the deferred gain at five years and a further 5% at seven (Form 8997 Part III).",
    ),
    (
        "EXPLAIN_QOF_N_INCLUDED_SHORT",
        "Short-term deferred gain included in 2026: the smaller of the deferred gain and the investment's value on the earlier of its sale and December 31, 2026, less the basis increase, shared by the character of the deferred gain (Form 8949 Part I box C, code Y).",
    ),
    (
        "EXPLAIN_QOF_N_INCLUDED_LONG",
        "Long-term deferred gain included in 2026, by the same rule (Form 8949 Part II box F, code Y).",
    ),
    (
        "EXPLAIN_QOF_N_INCLUDED_PROPERTY",
        "Section 1231 part of the deferred gain included in 2026 (special gain code B): Form 4797 Part I, with the Form 8949 code Y and code O rows netting to zero.",
    ),
    (
        "EXPLAIN_QOF_N_BASIS_AFTER",
        "Basis in the investment after the inclusion: the basis increase plus the gain included (Form 8997 Part V). Use it as the basis when you enter the sale of the investment.",
    ),
    (
        "EXPLAIN_QOF_DEFERRAL_N_SHORT",
        "Short-term 2026 gain deferred by an investment in a qualified opportunity fund in 2027 (Form 8997 Part II; Form 8949 Part I box C, code Z).",
    ),
    (
        "EXPLAIN_QOF_DEFERRAL_N_LONG",
        "Long-term 2026 gain deferred by an investment in a qualified opportunity fund in 2027 (Form 8997 Part II; Form 8949 Part II box F, code Z).",
    ),
    (
        "EXPLAIN_CAPITAL_GAIN_COLUMN_D",
        "Total of column (d), proceeds, of the Form 8949 rows on this Schedule D line.",
    ),
    (
        "EXPLAIN_CAPITAL_GAIN_COLUMN_E",
        "Total of column (e), cost or other basis, of the Form 8949 rows on this Schedule D line.",
    ),
    (
        "EXPLAIN_CAPITAL_GAIN_COLUMN_G",
        "Total of column (g), adjustments, of the Form 8949 rows on this Schedule D line.",
    ),
    ("LINE_SALE_N_MARKET_DISCOUNT", "Sale {0} market discount"),
    ("LINE_QOF_N_BASIS_INCREASE", "Opportunity fund {0} basis increase"),
    ("LINE_QOF_N_INCLUDED_SHORT", "Opportunity fund {0} short-term gain included"),
    ("LINE_QOF_N_INCLUDED_LONG", "Opportunity fund {0} long-term gain included"),
    ("LINE_QOF_N_INCLUDED_PROPERTY", "Opportunity fund {0} section 1231 gain included"),
    ("LINE_QOF_N_BASIS_AFTER", "Opportunity fund {0} basis after inclusion"),
    ("LINE_QOF_DEFERRAL_N_SHORT", "Opportunity fund deferral {0} short-term gain"),
    ("LINE_QOF_DEFERRAL_N_LONG", "Opportunity fund deferral {0} long-term gain"),
    ("LINE_FORM_N_N_N_PROCEEDS", "Form {0} entry {1} row {2} proceeds"),
    ("LINE_FORM_N_N_N_BASIS", "Form {0} entry {1} row {2} cost or other basis"),
    ("LINE_FORM_N_N_N_ADJUSTMENT", "Form {0} entry {1} row {2} adjustment"),
    ("LINE_FORM_N_N_N_GAIN", "Form {0} entry {1} row {2} gain or loss"),
    ("LINE_CAPITAL_GAIN_1A_PROCEEDS", "Capital gain 1a proceeds (d)"),
    ("LINE_CAPITAL_GAIN_1A_BASIS", "Capital gain 1a cost or other basis (e)"),
    ("LINE_CAPITAL_GAIN_1B_PROCEEDS", "Capital gain 1b proceeds (d)"),
    ("LINE_CAPITAL_GAIN_1B_BASIS", "Capital gain 1b cost or other basis (e)"),
    ("LINE_CAPITAL_GAIN_1B_ADJUSTMENT", "Capital gain 1b adjustments (g)"),
    ("LINE_CAPITAL_GAIN_8A_PROCEEDS", "Capital gain 8a proceeds (d)"),
    ("LINE_CAPITAL_GAIN_8A_BASIS", "Capital gain 8a cost or other basis (e)"),
    ("LINE_CAPITAL_GAIN_8B_PROCEEDS", "Capital gain 8b proceeds (d)"),
    ("LINE_CAPITAL_GAIN_8B_BASIS", "Capital gain 8b cost or other basis (e)"),
    ("LINE_CAPITAL_GAIN_8B_ADJUSTMENT", "Capital gain 8b adjustments (g)"),
    ("LINE_CAPITAL_GAIN_N_PROCEEDS", "Capital gain {0} proceeds (d)"),
    ("LINE_CAPITAL_GAIN_N_BASIS", "Capital gain {0} cost or other basis (e)"),
    ("LINE_CAPITAL_GAIN_N_ADJUSTMENT", "Capital gain {0} adjustments (g)"),
    ("LINE_QUALIFIED_NET_DISASTER_LOSS", "Qualified disaster loss, Schedule A line 17b"),
    ("LINE_EDUCATOR_ITEMIZED_DEDUCTION", "Educator and coach expenses, Schedule A line 17k"),
    (
        "EXPLAIN_EDUCATOR_ITEMIZED_DEDUCTION",
        "Eligible classroom expenses left after Schedule 1 line 11, other instructional expenses, and interscholastic sports administrator or coach expenses.",
    ),
    ("LINE_MEDICAL_EXPENSES", "Medical and dental expenses, Schedule A line 1"),
    (
        "EXPLAIN_MEDICAL_EXPENSES",
        "Eligible unreimbursed medical and dental expenses before the adjusted-gross-income floor.",
    ),
    ("LINE_MEDICAL_AGI", "Adjusted gross income, Schedule A line 2"),
    ("EXPLAIN_MEDICAL_AGI", "Adjusted gross income used for the medical expense deduction."),
    ("LINE_MEDICAL_FLOOR", "Medical expense floor, Schedule A line 3"),
    ("EXPLAIN_MEDICAL_FLOOR", "7.5% of adjusted gross income."),
    ("LINE_MEDICAL_DEDUCTION", "Medical deduction, Schedule A line 4"),
    ("EXPLAIN_MEDICAL_DEDUCTION", "Eligible medical expenses above the 7.5% floor."),
    ("LINE_SALT_INCOME_OR_SALES_TAX", "Income or sales taxes, Schedule A line 5a"),
    (
        "EXPLAIN_SALT_INCOME_OR_SALES_TAX",
        "The elected income or general sales taxes, including final-year estate and trust shares.",
    ),
    ("LINE_SALT_REAL_ESTATE_TAX", "Real estate taxes, Schedule A line 5b"),
    (
        "EXPLAIN_SALT_REAL_ESTATE_TAX",
        "Eligible real estate taxes, after moving an actual-expense home office share to the business.",
    ),
    ("LINE_SALT_PERSONAL_PROPERTY_TAX", "Personal property taxes, Schedule A line 5c"),
    (
        "EXPLAIN_SALT_PERSONAL_PROPERTY_TAX",
        "Annual value-based personal property taxes, including final-year estate and trust shares.",
    ),
    ("LINE_SALT_BEFORE_LIMIT", "Taxes before the limit, Schedule A line 5d"),
    (
        "EXPLAIN_SALT_BEFORE_LIMIT",
        "Income or sales taxes plus real estate and personal property taxes before the applicable limit.",
    ),
    ("LINE_SALT_DEDUCTION", "State and local tax deduction, Schedule A line 5e"),
    (
        "EXPLAIN_SALT_DEDUCTION",
        "Allowed state and local taxes, with investment real estate taxes not subject to the personal SALT limit.",
    ),
    ("LINE_OTHER_TAXES_DEDUCTION", "Other taxes, Schedule A line 6"),
    (
        "EXPLAIN_OTHER_TAXES_DEDUCTION",
        "Generation-skipping transfer tax on a taxable trust income distribution.",
    ),
    ("LINE_TAXES_DEDUCTION", "Tax deduction, Schedule A line 7"),
    ("EXPLAIN_TAXES_DEDUCTION", "State and local taxes plus other deductible taxes."),
    ("LINE_INTEREST_DEDUCTION", "Interest deduction, Schedule A line 10"),
    (
        "EXPLAIN_INTEREST_DEDUCTION",
        "Allowed home mortgage interest, points and premiums plus investment interest.",
    ),
    ("LINE_CHARITY_CASH_GIFTS", "Cash gifts, Schedule A line 11"),
    ("EXPLAIN_CHARITY_CASH_GIFTS", "Current cash gifts before the charitable limits and floor."),
    ("LINE_CHARITY_NONCASH_GIFTS", "Gifts other than cash, Schedule A line 12"),
    (
        "EXPLAIN_CHARITY_NONCASH_GIFTS",
        "Current noncash and other charitable gifts before the charitable limits and floor.",
    ),
    ("LINE_CHARITY_CURRENT_ALLOWED", "Current gifts allowed, Schedule A line 13"),
    (
        "EXPLAIN_CHARITY_CURRENT_ALLOWED",
        "Current gifts allowed after the charitable limits and floor.",
    ),
    ("LINE_CHARITY_CARRYOVER_ALLOWED", "Charitable carryovers allowed, Schedule A line 14"),
    ("EXPLAIN_CHARITY_CARRYOVER_ALLOWED", "Prior-year charitable gifts allowed this year."),
    ("LINE_CHARITY_DEDUCTION", "Charitable deduction, Schedule A line 15"),
    (
        "EXPLAIN_CHARITY_DEDUCTION",
        "Current and carried gifts allowed after the charitable limits and floor.",
    ),
    (
        "LINE_STANDARD_WITH_DISASTER_LOSS",
        "Standard deduction with disaster loss, Schedule A line 17c",
    ),
    (
        "EXPLAIN_STANDARD_WITH_DISASTER_LOSS",
        "The standard deduction claimed with a qualified net disaster loss; the loss itself is on line 17b.",
    ),
    ("LINE_BOND_PREMIUM_DEDUCTION", "Bond premium, Schedule A line 17f"),
    (
        "EXPLAIN_BOND_PREMIUM_DEDUCTION",
        "Eligible amortizable premium on taxable bonds entered from the bond records.",
    ),
    ("LINE_DEBT_INSTRUMENT_LOSS_DEDUCTION", "Debt instrument loss, Schedule A line 17g"),
    (
        "EXPLAIN_DEBT_INSTRUMENT_LOSS_DEDUCTION",
        "Ordinary loss on contingent payment or inflation-indexed debt from the issuer or broker records.",
    ),
    ("LINE_IMPAIRMENT_WORK_DEDUCTION", "Impairment-related work expenses, Schedule A line 17j"),
    (
        "EXPLAIN_IMPAIRMENT_WORK_DEDUCTION",
        "Eligible disability-related attendant care and other employee expenses needed to work.",
    ),
    ("LINE_OTHER_ITEMIZED_TOTAL", "Other itemized deductions, Schedule A line 17z"),
    (
        "EXPLAIN_OTHER_ITEMIZED_TOTAL",
        "The sum of Schedule A lines 17a through 17k, including qualified disaster losses and the standard deduction only on the increased-standard route.",
    ),
    ("LINE_ITEMIZED_DEDUCTIONS_ALLOWED", "Allowed itemized deductions, Schedule A line 18"),
    (
        "EXPLAIN_ITEMIZED_DEDUCTIONS_ALLOWED",
        "Total itemized deductions after the section 68 reduction.",
    ),
    ("LINE_MORTGAGE_GRANDFATHERED_BALANCE", "Grandfathered debt, Pub. 936 Table 1 line 1"),
    (
        "EXPLAIN_MORTGAGE_GRANDFATHERED_BALANCE",
        "Average qualified grandfathered debt under the monthly use-of-proceeds allocation.",
    ),
    ("LINE_MORTGAGE_PRE_N_BALANCE", "Pre-{0} acquisition debt, Pub. 936 Table 1 line 2"),
    (
        "EXPLAIN_MORTGAGE_PRE_2018_BALANCE",
        "Average acquisition debt incurred before December 16, 2017, retaining a refinance date only through the permitted term.",
    ),
    ("LINE_MORTGAGE_POST_N_BALANCE", "Post-{0} acquisition debt, Pub. 936 Table 1 line 7"),
    (
        "EXPLAIN_MORTGAGE_POST_2017_BALANCE",
        "Average later acquisition debt, including refinanced earlier debt after its permitted term.",
    ),
    ("LINE_MORTGAGE_QUALIFIED_INTEREST", "Qualified interest, Pub. 936 Table 1 line 13"),
    (
        "EXPLAIN_MORTGAGE_QUALIFIED_INTEREST",
        "Interest allocated to grandfathered and acquisition debt before Table 1 limits.",
    ),
    ("LINE_MORTGAGE_OTHER_USE_INTEREST", "Interest on money used for other things"),
    (
        "EXPLAIN_MORTGAGE_OTHER_USE_INTEREST",
        "Interest allocated to proceeds used for something other than buying, building or substantially improving the home; enter business, rental or investment interest on that activity.",
    ),
    ("LINE_MORTGAGE_NOT_QUALIFIED_HOME_INTEREST", "Interest on another home"),
    (
        "EXPLAIN_MORTGAGE_NOT_QUALIFIED_HOME_INTEREST",
        "Interest on a third home or a second home not chosen as a qualified home.",
    ),
    ("LINE_AMT_ITEMIZED_TAXES", "Taxes or standard deduction, Form 6251 line 2a"),
    (
        "EXPLAIN_AMT_ITEMIZED_TAXES",
        "State and local taxes, investment property taxes and foreign income taxes deducted on Schedule A, or the standard deduction, are added back for alternative minimum tax. Generation-skipping transfer tax on income distributions is excluded.",
    ),
    ("LINE_AMT_MORTGAGE_DWELLING_INTEREST", "Mortgage dwelling adjustment, Form 6251 line 3"),
    ("LINE_AMT_RELATED_ADJUSTMENTS", "Other adjustments, Form 6251 line 3"),
    ("LINE_AMT_SEP_DEDUCTION", "AMT self-employed SEP deduction"),
    ("LINE_AMT_SELF_EMPLOYED_HEALTH_DEDUCTION", "AMT self-employed health insurance deduction"),
    ("LINE_AMT_IRA_DEDUCTION", "AMT IRA deduction"),
    (
        "EXPLAIN_AMT_RELATED_ADJUSTMENTS",
        "Form 6251 line 3 combines mortgage adjustments and the regular-minus-AMT differences in income-limited deductions; each adjustment enters AMTI once.",
    ),
    (
        "EXPLAIN_AMT_RELATED_DEDUCTIONS",
        "The existing deduction worksheet uses actual AMT business income, unchanged actual half-SE tax and unchanged regular phaseout MAGI. This is an AMT deduction, not another payment or contribution.",
    ),
    (
        "EXPLAIN_AMT_MORTGAGE_DWELLING_INTEREST",
        "Deducted interest, points and premiums on a second home that is a boat or recreational vehicle are added back for alternative minimum tax.",
    ),
    (
        "MORTGAGE_RENTAL_POINTS_NEEDED",
        "Complete the loan term for de-minimis rental points, or enter this year's whole-loan deduction from its OID schedule, on Home mortgage interest.",
    ),
    (
        "HSA_ROLLOVER_BEFORE_RECEIPT",
        "The rollover cannot be deposited before the distribution was received.",
    ),
    (
        "HSA_ROLLOVER_PRECEDING_AFTER_RECEIPT",
        "The preceding rollover must be before this distribution.",
    ),
    ("SAVING_KIND_FARM_CHOICE_ONE", "Farm choice"),
    ("SAVING_KIND_FARM_CHOICE_MANY", "Farm choices"),
    ("SA1099_PAYMENT_RECORDS_NEEDED", "Complete this medical-account payment's records."),
    (
        "SA1099_MEDICARE_SPOUSAL_RECORDS_NEEDED",
        "Answer whether this Medicare MSA was inherited from your spouse.",
    ),
    (
        "SA1099_MEDICARE_PRIOR_RECORDS_NEEDED",
        "Complete the prior-year records for your own Medicare MSA.",
    ),
    ("Q1099_BENEFICIARY_NEEDED", "Identify the student for this education-account distribution."),
    ("Q1099_COVERDELL_BASIS_NEEDED", "Enter the opening Coverdell basis from your records."),
    (
        "Q1099_LATE_ROLLOVERS_NEEDED",
        "Enter the actual late-rollover adjustment, including zero when none.",
    ),
    ("Q1099_ROTH_RECORDS_NEEDED", "Complete the 529-to-Roth transfer records."),
    (
        "WARN_LTC_PERIODS_MISSING",
        "{0}: no Long-term care periods record fits this form, so its per diem payments are taxed in full; add the record for the insured person and your tax may be lower.",
    ),
    ("LINE_LTC_BENEFIT_N_NONQUALIFIED_EXCLUDED", "LTC benefit {0} nonqualified excluded"),
    ("LINE_LTC_BENEFIT_N_NONQUALIFIED_TAXABLE", "LTC benefit {0} nonqualified taxable"),
    (
        "EXPLAIN_LTC_NONQUALIFIED_TREATMENT",
        "Actual accident/health funding and eligible employer-attributable reimbursements or permanent-injury payments, outside Form 8853 SectionC’s per-diem limit.",
    ),
    ("LINE_ARCHER_MSA_N_EXCESS_EARNINGS", "Archer MSA {0} excess earnings"),
    ("LINE_INHERITED_MEDICAL_N_EARNINGS", "Inherited medical {0} earnings after the death"),
    ("LINE_INHERITED_MEDICAL_EARNINGS", "Inherited medical earnings after the death"),
    (
        "EXPLAIN_ARCHER_MSA_N_EXCESS_EARNINGS",
        "Form 1099-SA box 2 earnings on a returned Archer MSA excess contribution. The returned contribution itself is not income; its earnings are Schedule 1 line 8z other income, with no additional tax.",
    ),
    (
        "EXPLAIN_INHERITED_MEDICAL_EARNINGS",
        "Earnings after the account holder's death: Form 1099-SA box 1 above the box 4 value at death. The 1099-SA recipient instructions put them on Schedule 1 line 8z other income in the year received.",
    ),
    ("LINE_ROYALTY_N_LINE13_INTEREST", "Royalty {0,number,0}: interest allowed"),
    ("LINE_AMT_INVESTMENT_INTEREST_ADJUSTMENT", "AMT investment-interest adjustment"),
    ("LINE_INVESTMENT_SOURCE_N_ALLOWED", "Investment source {0,number,0}: interest allowed"),
    ("LINE_INVESTMENT_SOURCE_N_CARRYFORWARD", "Investment source {0,number,0}: carryforward"),
    (
        "LINE_INVESTMENT_SOURCE_N_AMT_ALLOWED",
        "Investment source {0,number,0}: AMT interest allowed",
    ),
    (
        "LINE_INVESTMENT_SOURCE_N_AMT_CARRYFORWARD",
        "Investment source {0,number,0}: AMT carryforward",
    ),
    ("LINE_AMT_INVESTMENT_FORM_LINE1", "AMT Form 4952 line 1: current interest"),
    ("LINE_AMT_INVESTMENT_FORM_LINE2", "AMT Form 4952 line 2: prior disallowed interest"),
    ("LINE_AMT_INVESTMENT_FORM_LINE3", "AMT Form 4952 line 3: total interest"),
    ("LINE_AMT_INVESTMENT_FORM_LINE4A", "AMT Form 4952 line 4a: gross investment income"),
    ("LINE_AMT_INVESTMENT_FORM_LINE4B", "AMT Form 4952 line 4b: qualified dividends"),
    ("LINE_AMT_INVESTMENT_FORM_LINE4C", "AMT Form 4952 line 4c: ordinary investment income"),
    ("LINE_AMT_INVESTMENT_FORM_LINE4D", "AMT Form 4952 line 4d: investment disposition gain"),
    ("LINE_AMT_INVESTMENT_FORM_LINE4E", "AMT Form 4952 line 4e: net investment capital gain"),
    ("LINE_AMT_INVESTMENT_FORM_LINE4F", "AMT Form 4952 line 4f: nonpreferential investment gain"),
    ("LINE_AMT_INVESTMENT_FORM_LINE4G", "AMT Form 4952 line 4g: preferential-income election"),
    ("LINE_AMT_INVESTMENT_FORM_LINE4H", "AMT Form 4952 line 4h: investment income"),
    ("LINE_AMT_INVESTMENT_FORM_LINE5", "AMT Form 4952 line 5: allowed investment expenses"),
    ("LINE_AMT_INVESTMENT_FORM_LINE6", "AMT Form 4952 line 6: net investment income"),
    ("LINE_AMT_INVESTMENT_FORM_LINE7", "AMT Form 4952 line 7: interest carried forward"),
    ("LINE_AMT_INVESTMENT_FORM_LINE8", "AMT Form 4952 line 8: interest allowed"),
    ("LINE_INVESTMENT_FORM_LINE1", "Form 4952 line 1: current interest"),
    ("LINE_INVESTMENT_FORM_LINE2", "Form 4952 line 2: prior disallowed interest"),
    ("LINE_INVESTMENT_FORM_LINE3", "Form 4952 line 3: total interest"),
    ("LINE_INVESTMENT_FORM_LINE4A", "Form 4952 line 4a: gross investment income"),
    ("LINE_INVESTMENT_FORM_LINE4B", "Form 4952 line 4b: qualified dividends"),
    ("LINE_INVESTMENT_FORM_LINE4C", "Form 4952 line 4c: ordinary investment income"),
    ("LINE_INVESTMENT_FORM_LINE4D", "Form 4952 line 4d: investment disposition gain"),
    ("LINE_INVESTMENT_FORM_LINE4E", "Form 4952 line 4e: net investment capital gain"),
    ("LINE_INVESTMENT_FORM_LINE4F", "Form 4952 line 4f: nonpreferential investment gain"),
    ("LINE_INVESTMENT_FORM_LINE4G", "Form 4952 line 4g: preferential-income election"),
    ("LINE_INVESTMENT_FORM_LINE4H", "Form 4952 line 4h: investment income"),
    ("LINE_INVESTMENT_FORM_LINE5", "Form 4952 line 5: allowed investment expenses"),
    ("LINE_INVESTMENT_FORM_LINE6", "Form 4952 line 6: net investment income"),
    ("LINE_INVESTMENT_FORM_LINE7", "Form 4952 line 7: interest carried forward"),
    ("LINE_INVESTMENT_FORM_LINE8", "Form 4952 line 8: interest allowed"),
    (
        "EXPLAIN_INVESTMENT_SOURCE_ROUTING",
        "One Form 4952 limit uses gross investment income and allowed noninterest expenses. Determined source amounts go once to Schedule A or Schedule E, with distinct regular and AMT carryforwards; no priority is guessed between limited mixed sources.",
    ),
    (
        "INVESTMENT_BORROWING_RECORDS",
        "Check each borrowing amount and choose its current income source. Reopen this page if the income sources changed.",
    ),
    (
        "RECOVERY_DEDUCTED_ABOVE_ITEMIZED",
        "What you deducted for these items cannot be more than that year's total itemized deductions.",
    ),
    ("MISC_RENTS_NEED_A_USE", "Say where the rents in box 1 belong."),
    ("MISC_ROYALTIES_NEED_A_USE", "Say where the royalties in box 2 belong."),
    ("MISC_OTHER_NEEDS_A_USE", "Say where the other income in box 3 belongs."),
    ("FORM_1099G_GRANT_NEEDS_A_USE", "Say who received the box 6 grant."),
    (
        "ISSUE_RECOVERY_REFUND_YEAR_FACTS_DIFFER",
        "Recoveries of the same year's deductions must use the same itemized deductions, standard deduction and taxable income from that year's return.",
    ),
    (
        "WARN_FORM_1099_DESTINATION_LEFT_OUT",
        "{0}: the business, rental, royalty, equipment or hobby entry that receives amounts from this form is left out of the estimate, so those amounts are left out too; your tax may be higher.",
    ),
    (
        "WARN_RECOVERY_LEFT_OUT",
        "{0}: a recovery that needs the earlier year's tax refigured (unused credits, alternative minimum tax, the 0% capital gain rate, a dependent filer, or an earlier refund of the same deduction) isn't calculated, so it is left out of this estimate; your tax may be higher.",
    ),
    ("LINE_ALASKA_DIVIDENDS", "Alaska Permanent Fund dividends"),
    (
        "EXPLAIN_ALASKA_DIVIDENDS",
        "Alaska Permanent Fund dividends, Schedule 1 line 8g: the whole dividend, including any energy relief part. Unearned income for the child tax rules; not net investment income, earned income or self-employment income.",
    ),
    ("LINE_OLYMPIC_PRIZES", "Olympic and Paralympic prizes"),
    (
        "EXPLAIN_OLYMPIC_PRIZES",
        "Value of Olympic and Paralympic medals and USOC prize money on Schedule 1 line 8m. Line 24c removes it when adjusted gross income without the exclusion is $1,000,000 or less ($500,000 married filing separately).",
    ),
    ("LINE_OLYMPIC_EXCLUSION", "Olympic and Paralympic exclusion"),
    (
        "EXPLAIN_OLYMPIC_EXCLUSION",
        "Schedule 1 line 24c: the line 8m prize excluded under section 74(d) because adjusted gross income without the exclusion is not above $1,000,000 ($500,000 married filing separately). Zero when the prize is taxable.",
    ),
    ("LINE_DEFERRED_PENSION", "Nonqualified plan pension"),
    (
        "EXPLAIN_DEFERRED_PENSION",
        "W-2 box 1 pension or annuity from a nonqualified deferred compensation plan or a nongovernmental section 457 plan, Schedule 1 line 8t instead of Form 1040 line 1a. Not earned income for the earned income credit or compensation for an IRA; earned income for the dependent standard deduction.",
    ),
    ("LINE_INMATE_PAY", "Pay while an inmate"),
    (
        "EXPLAIN_INMATE_PAY",
        "Pay for work while an inmate in a penal institution, Schedule 1 line 8u instead of Form 1040 line 1a. Not earned income for the earned income credit; compensation for an IRA and earned income for the dependent standard deduction.",
    ),
    ("LINE_DIGITAL_ASSET_INCOME", "Digital assets received as income"),
    (
        "EXPLAIN_DIGITAL_ASSET_INCOME",
        "Fair market value of digital assets received as ordinary income (staking, mining not as a business, airdrops, hard forks), Schedule 1 line 8v. That value is the basis of a later sale; not net investment income or self-employment income.",
    ),
    ("LINE_FORM_1099K_NOT_INCOME", "Form 1099-K amounts that were not income"),
    (
        "EXPLAIN_FORM_1099K_NOT_INCOME",
        "Form 1099-K amounts included in error (gifts, shared costs) or for personal items sold at a loss, entered at the top of Schedule 1. Not taxed, and a personal-use loss is not deductible.",
    ),
    ("LINE_OTHER_INCOME_N_CARD_INSURANCE", "Other income {0} credit card insurance"),
    ("LINE_OTHER_INCOME_N_OTHER", "Other income {0}"),
    ("LINE_OTHER_INCOME_N_RTAA", "Other income {0} RTAA payments"),
    ("LINE_OTHER_INCOME_N_SUBSTITUTE_PAYMENTS", "Other income {0} substitute payments"),
    ("LINE_OTHER_INCOME_N_UNEMPLOYMENT_FUND", "Other income {0} unemployment fund benefits"),
    ("LINE_RECOVERY_N_TAXABLE", "Recovery of {0} deductions taxable"),
    (
        "EXPLAIN_OTHER_INCOME_N_CARD_INSURANCE",
        "Credit card disability or unemployment insurance benefits above the premiums paid in 2026, Schedule 1 line 8z (Publication 525).",
    ),
    (
        "EXPLAIN_OTHER_INCOME_N_UNEMPLOYMENT_FUND",
        "Private or union unemployment fund benefits above your own nondeductible payments into the fund, Schedule 1 line 8z (Publication 525).",
    ),
    (
        "EXPLAIN_OTHER_INCOME_N_OTHER",
        "{0}: other taxable income on Schedule 1 line 8z. Not self-employment income or earned income.",
    ),
    (
        "EXPLAIN_OTHER_INCOME_N_RTAA",
        "Form 1099-G box 5 reemployment trade adjustment assistance (RTAA) payments, Schedule 1 line 8z.",
    ),
    (
        "EXPLAIN_OTHER_INCOME_N_SUBSTITUTE_PAYMENTS",
        "Form 1099-MISC box 8 substitute payments in lieu of dividends or interest, Schedule 1 line 8z. Not qualified dividends or tax-exempt interest; net investment income (Form 8960 line 7).",
    ),
    (
        "EXPLAIN_RECOVERY_N_TAXABLE",
        "Publication 525 Worksheet 2 line 11 for this prior year: the part of a refund or reimbursement of an itemized deduction that lowered that year's tax, limited by the deduction, by the itemized deductions above the standard deduction and by a negative taxable income. The state or local income tax refund share is on Schedule 1 line 1; the rest is on line 8z.",
    ),
    ("LINE_BUSINESS_N_FROM_FORMS", "Business {0} receipts from Forms 1099"),
    (
        "EXPLAIN_BUSINESS_N_FORM_1099_RECEIPTS",
        "Part of Schedule C line 1: the Form 1099-NEC, 1099-K and 1099-MISC amounts entered on Self-employment and Other income that name this business, less a 1099-K part's refunds and the part of box 10 paid on to clients. The business's own typed receipts leave these forms out.",
    ),
    (
        "WARN_FARM_CCC_LEFT_OUT",
        "{0}: CCC loans with special treatment aren't supported, so they are left out of this estimate; your tax may be higher.",
    ),
    (
        "WARN_MOVING_LEFT_OUT",
        "A move that was not under military orders for a permanent change of station or an intelligence-community reassignment is not deductible (section 217(k)), so it is left out of this estimate.",
    ),
    (
        "WARN_ISO_SALE_LEFT_OUT",
        "{0}: special treatment of these ISO shares isn't supported, so this sale is left out of this estimate; your tax may be higher or lower.",
    ),
    (
        "WARN_SCORP_AMT_LEFT_OUT",
        "{0}: AMT differences beyond the entered records aren't supported, so AMT uses this S corporation's regular-tax records.",
    ),
    (
        "WARN_MINIMUM_TAX_CREDIT_NOT_CLAIMED",
        "Assumed your prior-year minimum tax credit history is incomplete, so the credit (Form 8801) is not claimed; if it is complete, your tax may be lower.",
    ),
    (
        "WARN_BBA_AUDIT_LEFT_OUT",
        "Form 8978 cannot be figured until the prior-return records and applicable confirmations are supplied. Its tax adjustment is left out of this estimate; supply the missing records in More credits.",
    ),
    (
        "WARN_TAX_CREDIT_BOND_LEFT_OUT",
        "{0}: this bond's credit cannot be established from the available bond records and eligibility facts. The credit is left out; independently known income is retained.",
    ),
    (
        "WARN_VOLUNTEER_EXPENSE_LEFT_OUT",
        "{0}: volunteer expenses not confirmed as eligible and documented aren't deducted.",
    ),
    (
        "WARN_WHALING_EXPENSES_LEFT_OUT",
        "Whaling captain expenses not confirmed as eligible and documented aren't deducted.",
    ),
    (
        "WARN_CARE_COSTS_LEFT_OUT",
        "{0}: isn't a Form 2441 qualifying person (a dependent under 13, or a disabled dependent or spouse who can't care for themselves and lived with you over half the year), so the care costs entered for them are left out of this estimate; if they are one, correct their details on Children and people you support or their answer here, and your tax may be lower.",
    ),
    (
        "WARN_FARM_LAND_SALE_LEFT_OUT",
        "{0}: special treatment of this farm-land sale isn't supported, so the sale is left out of this estimate; your tax may be higher.",
    ),
    (
        "WARN_HOME_SALE_LEFT_OUT",
        "{0}: special treatment of this home sale isn't supported, so the sale is left out of this estimate; your tax may be higher.",
    ),
    (
        "WARN_HSA_FUNDING_LEFT_OUT",
        "An IRA-to-HSA funding transfer is outside the supported basis and source-date treatment, so its exclusion is left out and its source follows the ordinary IRA rules; your tax may be lower.",
    ),
    (
        "WARN_PARTNERSHIP_K1_LEFT_OUT",
        "{0}: special items, basis or financing on this partnership K-1 aren't supported, so the K-1 is left out of this estimate; your tax may be higher or lower.",
    ),
    (
        "WARN_PARTNER_FUEL_CREDIT_LEFT_OUT",
        "{0}: this K-1's fuel tax credit statement or farm use isn't confirmed, so the fuel tax credit is left out of this estimate; your tax may be lower.",
    ),
    (
        "WARN_PARTNER_FUEL_RECOVERY_LEFT_OUT",
        "{0}: a prior fuel credit recovery not claimed on a 2025 return filed in 2026 by a cash-method activity isn't supported, so it is left out of this estimate; your tax may be higher.",
    ),
    (
        "WARN_PARTNER_FUEL_PARTIAL_BENEFIT_LEFT_OUT",
        "{0}: a partial prior tax benefit from the fuel cost isn't supported, so the full credited fuel tax is included as recovered income; your tax may be lower.",
    ),
    (
        "WARN_PARTNER_FUEL_NIIT_RECOVERY_LEFT_OUT",
        "{0}: recovering a fuel-cost deduction that reduced net investment income tax isn't supported, so that recovery is left out of net investment income; your tax may be higher.",
    ),
    (
        "WARN_MARKETPLACE_MIXED_MONTH_LEFT_OUT",
        "Months when only some covered people qualified for the premium tax credit aren't supported, so those months are left out of the credit and their advance payments are repaid; your tax may be lower.",
    ),
    (
        "WARN_NOL_LEFT_OUT",
        "A net operating loss with an investment, trading or excess business loss part, a farm carryback, a foreign, inherited or trust loss, a change of filer or an unsettled earlier return isn't supported, so the regular loss carryforward is left out of this estimate; your tax may be lower.",
    ),
    (
        "WARN_PROPERTY_ROLLOVER_LEFT_OUT",
        "{0}: distributed property whose complete sale and ordinary investment treatment aren't confirmed isn't supported, so this distribution is left out of this estimate; your tax may be higher.",
    ),
    (
        "WARN_RETIREMENT_BASIS_LEFT_OUT",
        "{0}: this distribution's special basis-recovery treatment isn't confirmed, so it isn't supported and no after-tax cost is recovered; your tax may be lower.",
    ),
    (
        "WARN_ADOPTION_CARRY_LEFT_OUT",
        "An adoption credit carryforward not confirmed as your available credit from an eligible original return isn't supported, so it is left out of this estimate; your tax may be lower.",
    ),
    (
        "WARN_REPAYMENT_LEFT_OUT",
        "Claim-of-right eligibility for your earlier-year repayment isn't confirmed, so the repayment gets no deduction or credit in this estimate; if it qualifies, your tax may be lower.",
    ),
    (
        "WARN_REPAYMENT_CREDIT_LEFT_OUT",
        "Some original-year records for the repayment comparison aren't supported, so only the itemized deduction is figured, not the claim-of-right credit; your tax may be lower.",
    ),
    (
        "WARN_COOPERATIVE_LEFT_OUT",
        "{0}: this Form 1099-PATR isn't an ordinary statement for a farm calculated on this return, so it is left out of this estimate; your tax may be higher.",
    ),
    (
        "WARN_COOPERATIVE_PROPERTY_PART_TAXED",
        "{0}: patronage dividends from buying capital assets or depreciable property aren't figured as basis reductions, so they are included in farm income; your tax may be lower.",
    ),
    (
        "WARN_COOPERATIVE_CREDITS_LEFT_OUT",
        "{0}: the credits and deductions in boxes 10–12 aren't calculated, so they are left out of this estimate; your tax may be lower.",
    ),
    (
        "WARN_FARM_RENTAL_LOSS_LEFT_OUT",
        "{0}: part of this farm rental's investment is not at risk (Form 4835 line 34b) and the Form 6198 limit isn't calculated, so its loss is left out of this estimate; if the loss is allowed, your tax may be lower.",
    ),
    (
        "WARN_FARM_RENTAL_SECTION_179_LEFT_OUT",
        "{0}: section 179 for property of a farm rental isn't calculated, so the property is depreciated instead; your tax may be lower if you qualify for section 179.",
    ),
    (
        "WARN_FARM_AVERAGING_LEFT_OUT",
        "Farm income averaging with a special situation isn't supported, so Schedule J isn't used and tax is figured the regular way; your tax may be lower.",
    ),
    (
        "WARN_INTEREST_MARKET_DISCOUNT_LEFT_OUT",
        "{0}: interest with market discount (box 10) isn't supported, so this statement is left out of this estimate; your tax may be higher.",
    ),
    (
        "WARN_FOREIGN_HOUSING_LEFT_OUT",
        "{0}: this foreign housing claim's eligibility, election or household facts aren't confirmed, so no housing exclusion or deduction is figured for it; your tax may be lower.",
    ),
    (
        "WARN_HOBBY_SALES_LEFT_OUT",
        "{0}: sales that aren't hobby goods sales aren't figured here, so they are left out of this estimate; enter a business on Self-employment and collectibles or other personal items sold at a gain on Investment sales, or your tax may be higher.",
    ),
    (
        "EXPLAIN_ADOPTION_W2_N_CODE_T_TAXABLE",
        "The box 12 code T adoption benefits on {0} that no adoption on More credits accounts for. They are taxable wages on Form 1040 line 1f (Form 8839 Part III).",
    ),
    (
        "EXPLAIN_EDUCATION_N_SCHOLARSHIP_INCLUDED",
        "Part of the box 5 grant that {0} includes in income because it pays living costs (Pub. 970 chapter 2): the education credit uses more of the tuition, and this part is taxable scholarship.",
    ),
    (
        "WARN_QSBS_SALE_LEFT_OUT",
        "{0}: qualified small business stock not bought for cash at original issue, or with a changed basis, isn't supported, so this sale is left out of this estimate; your tax may be higher.",
    ),
    (
        "WARN_SCORP_K1_LEFT_OUT",
        "{0}: special items, basis or financing on this S corporation K-1 aren't supported, so the K-1 is left out of this estimate; your tax may be higher or lower.",
    ),
    (
        "WARN_PRIOR_STOCK_CARRY_LEFT_OUT",
        "A 2025 public-stock charity carryover with a changed filing unit or incomplete records isn't supported, so it is left out of this estimate; your tax may be lower.",
    ),
    (
        "WARN_SMALL_BUSINESS_STOCK_LEFT_OUT",
        "{0}: this small business stock loss's ownership or basis history isn't supported, so the loss is left out of this estimate; your tax may be lower.",
    ),
    (
        "WARN_HOSTED_STUDENT_LEFT_OUT",
        "{0}: the sponsor, student or reimbursement facts for this hosted student aren't confirmed, so the deduction is left out of this estimate; your tax may be lower.",
    ),
    // Answers and unknowns.
    (
        "WARN_ATNOL_LEFT_OUT",
        "An alternative minimum tax loss with an investment, trading or excess business loss part, a farm carryback, a foreign, inherited or trust loss, a change of filer or an unsettled earlier return isn't supported, so the AMT loss carryforward is left out of this estimate; your tax may be lower.",
    ),
    (
        "WARN_ESPP_SALE_LEFT_OUT",
        "{0}: an employee stock purchase plan sale with special treatment isn't supported, so this sale is left out of this estimate; your tax may be higher.",
    ),
    (
        "WARN_RENTAL_LEFT_OUT",
        "{0}: a special rental situation, a property received in a like-kind exchange before 2026 (its depreciation continues the property given up, Treas. Reg. 1.168(i)-6) isn't calculated (Schedule E Part I), so this rental is left out of this estimate; your tax may be higher, or lower if it had a loss.",
    ),
    (
        "WARN_QBI_WAGES_LEFT_OUT",
        "{0}: W-2 wages not confirmed as attributable to this business aren't counted toward the qualified business income wage limit; if they are, your tax may be lower.",
    ),
    (
        "WARN_FOREIGN_PRESENCE_TEST_NOT_MET",
        "{0}: {1,number,integer} full days abroad is fewer than the 330 the physical presence test needs, so no foreign exclusion or housing amount is figured.",
    ),
    (
        "WARN_FORM_2555_NO_FOREIGN_INCOME",
        "{0}: this Form 2555 has no foreign earned income to exclude and no 2025 housing deduction to carry, so it claims nothing. Mark W-2 pay for work abroad on its W-2, enter pay from a foreign employer, or mark a business run abroad.",
    ),
    (
        "WARN_K1_FOREIGN_TAX_WITHOUT_K3",
        "No credit is figured for {0}'s foreign taxes without its Schedule K-3; the K-1 box is not used for Form 1116. Ask the entity for a Schedule K-3 and enter it on this K-1.",
    ),
    (
        "WARN_K3_FOREIGN_TAX_NOT_CLAIMED",
        "{0}'s Schedule K-3 foreign taxes are not claimed; choose the foreign tax credit or an itemized foreign-tax deduction on More credits to claim them.",
    ),
    (
        "WARN_K3_FOREIGN_TAX_CREDIT_NOT_CLAIMED",
        "{0}'s Schedule K-3 reports items this estimate does not figure, so no foreign tax credit is claimed; if one applies, your tax may be lower.",
    ),
    (
        "WARN_LUMP_SUM_LEFT_OUT",
        "{0}: the conditions for taxing this lump sum separately on Form 4972 are not met (all conditions confirmed, the entire balance paid in 2026, and a qualifying event with no later earlier-year payment), so it is taxed with your other income; your tax may be higher.",
    ),
    (
        "NOTE_LUMP_SUM_PARTICIPANT_BORN_AFTER_1935",
        "{0}: Form 4972 is only for a plan participant born before January 2, 1936, so this lump sum is taxed with your other income. A spouse or former spouse paid under a qualified domestic relations order is taxed this way unless that person was born before January 2, 1936 and files their own return.",
    ),
    (
        "WARN_CAPITAL_LOSS_CARRYOVER_LEFT_OUT",
        "A prior-year capital loss carryover with an allocation to a former spouse, a canceled-debt reduction or another special adjustment isn't supported, so it is left out of this estimate; your tax may be lower.",
    ),
    (
        "WARN_INVENTORY_LEFT_OUT",
        "{0}: inventory with LIFO or another special valuation isn't supported, so no cost of goods sold is deducted in this estimate; your tax may be lower.",
    ),
    (
        "WARN_HOME_OFFICE_LEFT_OUT",
        "{0}: a home office with daycare use, a separate structure or another special case isn't calculated, so no home-office deduction is taken in this estimate; your tax may be lower.",
    ),
    (
        "WARN_FARM_SUPPLIES_LEFT_OUT",
        "{0}: an unused-supply payment is a deposit, has unestablished purchase terms or materially distorts income. That payment is not deducted this year; its net cost remains available for its actual later consumption. Other eligible supply records are retained.",
    ),
    (
        "WARN_FUEL_SALE_LEFT_OUT",
        "A fuel sale not confirmed as qualifying (its tax rate, buyer use, certificate or waiver, or that no one else claimed the gallons) is left out of the fuel tax credit; if it qualifies, your tax may be lower.",
    ),
    (
        "WARN_FUEL_CLAIM_LEFT_OUT",
        "A registered vendor's or card issuer's fuel claim whose registration, records or accounting isn't confirmed is left out of the fuel tax credit; if it qualifies, your tax may be lower.",
    ),
    (
        "WARN_SE_HEALTH_LEFT_OUT",
        "{0}: these entity-plan premiums are not deducted as self-employed health insurance because qualifying entity payment and income reporting are absent or unconfirmed. Independently eligible medical premiums remain available for itemization; review the existing K-1 or W-2 reporting before changing this answer.",
    ),
    (
        "MEDICAL_ADOPTED_CHILD_RELATIONSHIP",
        "The adopted-child medical exception applies to your own adopted child or a child lawfully placed with you for adoption. Review this person's relationship.",
    ),
    (
        "MEDICAL_PARENT_SUPPORT_FACTS_CONFLICT",
        "The separated-parent medical facts require your child, over-half combined parental support, and no multiple support agreement. Review the relationship and support answers.",
    ),
    (
        "WARN_LTC_PREMIUM_LEFT_OUT",
        "{0}: long-term-care premiums not confirmed as a qualified contract for an eligible person are left out of this estimate; if they qualify, your tax may be lower.",
    ),
    (
        "WARN_IRD_ESTATE_TAX_NOT_DEDUCTED",
        "The deduction for estate tax on inherited IRA income isn't taken because the estate figures aren't confirmed final; if they are, your tax may be lower.",
    ),
    (
        "WARN_INHERITED_IRA_RECORD_MISSING",
        "{0}: no Inherited IRA record fits this payment, so the amount the payer printed in box 2a is taxed with no basis; if the person you inherited it from left after-tax basis, your tax may be lower.",
    ),
    (
        "WARN_INHERITED_ROTH_BASIS_ASSUMED_NONE",
        "{0}: no basis was entered for this inherited Roth IRA and the owner's five years aren't over, so every payment is taxed as earnings; if it holds contributions, your tax may be lower.",
    ),
    (
        "WARN_K1_QBI_NO_STATEMENT",
        "{0}: this K-1 has no current section 199A statement, so it adds no current qualified business income to the QBI deduction. Retained losses keep their original QBI status and count when allowed. A statement reporting qualified income may lower your tax.",
    ),
    (
        "WARN_K1_QBI_LOSS_NO_STATEMENT",
        "{0}: this K-1 has no current section 199A statement, so its current business loss is left out of the QBI deduction. Retained losses keep their original QBI status and count when allowed. Including a qualified current loss can reduce the deduction from your other businesses; your tax may be higher.",
    ),
    (
        "WARN_TAX_REFUND_LEFT_OUT",
        "{0}: a state or local tax refund that needs the full tax-benefit worksheet or a multi-year allocation isn't calculated, so it is left out of this estimate; your tax may be higher.",
    ),
    // Answers and unknowns.
    (
        "WARN_UNDISTRIBUTED_GAIN_LEFT_OUT",
        "{0}: a Form 2439 statement not confirmed as your own taxable-account statement for a fund year ending in 2026 is left out of this estimate; if it is yours, your tax may be higher, or lower because of its tax-paid credit.",
    ),
    (
        "WARN_FINAL_BONDS_LEFT_OUT",
        "{0}: savings bonds held at death that aren't confirmed as complete, solely owned EE or I holdings without special treatment aren't supported, so their interest is left out of this final return; your tax may be higher.",
    ),
    (
        "ISSUE_AFTER_DEATH_WITHOUT_A_DEATH",
        "{0}: a part paid after a death applies only to a person who died in 2026; clear it, or enter the date of death on their page.",
    ),
    (
        "COMMUNITY_ANSWER_NOT_ALLOWED",
        "Choose Community, Separate or Partly community on a community-property separate return; any other return doesn't ask.",
    ),
    (
        "ESPP_ACQUISITION_NEEDED",
        "Supply the acquisition records for these ESPP shares in Investment sales.",
    ),
    (
        "ESPP_FIXED_GRANT_PRICE_NEEDED",
        "Form 3922 box 8 is blank. Supply the actual fixed grant-date exercise price from the option records.",
    ),
    (
        "ESPP_WAGE_INCLUSION_NEEDED",
        "Does W-2 box 1 already include the compensation from this ESPP sale?",
    ),
    (
        "ISO_SALE_ADJUSTMENTS_NEEDED",
        "Enter any selling fees, exercise commissions, option cost or special treatment for these ISO shares in Investment sales.",
    ),
    (
        "ISO_WAGE_INCLUSION_NEEDED",
        "Does W-2 box 1 already include the compensation from this ISO sale?",
    ),
    (
        "ISO_SALE_ANSWERS_NEEDED",
        "{0}: answer the questions about this ISO sale in Investment sales; until then it is left out of this estimate.",
    ),
    (
        "NUA_SALE_ADJUSTMENTS_NEEDED",
        "Enter any wash-sale adjustment or special treatment for these employer shares in Investment sales.",
    ),
    (
        "NUA_OWNERSHIP_NEEDED",
        "Were these employer shares your own, or inherited from someone who died before the sale?",
    ),
    (
        "NUA_SALE_ANSWERS_NEEDED",
        "{0}: answer the questions about this employer-stock sale in Investment sales; until then it is left out of this estimate.",
    ),
    (
        "RRB_EARLIER_YEAR_ALLOCATION_NEEDED",
        "RRB-1099 box 9 combines earlier years without a year-by-year breakdown. The estimate uses the regular method until actual year details are available; no earlier year is inferred.",
    ),
    ("DOCUMENT_NAMES_CHANGED", "Reopen this document before saving."),
    (
        "NOTE_FORM_8958_ALLOCATION",
        "Attach Form 8958 to each separate return. It lists each item's total and each spouse's share: line 1 wages, line 2 interest, line 3 dividends and the other lines for the other kinds, and line 11 federal income tax withheld. The employee's own Social Security and Medicare wages and tax, a business owner's self-employment tax, IRA compensation and earned income credit earnings are not divided.",
    ),
    (
        "EXPLAIN_FORM8958_N_TOTAL",
        "Form 8958 column A: the complete amount of {0} before the community-property division.",
    ),
    (
        "EXPLAIN_FORM8958_N_THIS_RETURN",
        "Form 8958 columns B and C: this return's share of {0} after the Community or separate answer; the owner keeps an odd cent so the two columns add to column A.",
    ),
    (
        "EXPLAIN_FORM8958_WITHHOLDING_N_TOTAL",
        "Form 8958 line 11 column A: the complete federal income tax withheld on {0} before the community-property division.",
    ),
    (
        "EXPLAIN_FORM8958_WITHHOLDING_N_THIS_RETURN",
        "Form 8958 line 11 columns B and C: this return's share of the federal income tax withheld on {0}, divided like its income.",
    ),
    ("LINE_FORM8958_N_TOTAL", "Form 8958 item {0} total"),
    ("LINE_FORM8958_N_THIS_RETURN", "Form 8958 item {0}, this return"),
    ("LINE_FORM8958_WITHHOLDING_N_TOTAL", "Form 8958 withholding {0} total"),
    ("LINE_FORM8958_WITHHOLDING_N_THIS_RETURN", "Form 8958 withholding {0}, this return"),
    (
        "NOTE_COMMUNITY_ITEM_IS_THE_SPOUSES",
        "{0}: this item belongs wholly to your spouse under your Community or separate answer, so it is not on this return.",
    ),
    (
        "WARN_SECTION_1256_LEFT_OUT",
        "{0}: section 1256 contracts held as a business or farm hedge, under a section 475(f) or 988 election, or as a dealer aren't supported, so this statement is left out of this estimate; your tax may be higher, or lower if it had a loss.",
    ),
    (
        "WARN_UNEMPLOYMENT_LEFT_OUT",
        "{0}: unemployment with employee contributions, private or union funds, supplemental employer pay, a treaty exclusion or unreconciled records isn't supported, so this statement is left out of this estimate; your tax may be higher.",
    ),
    (
        "WARN_RENTAL_SALE_PROPERTY_RECEIVED",
        "{0}: its Form 1099-S box 6 shows services or property received besides cash, notes or digital assets, but the rental is entered as sold. If you exchanged it for like-kind real property, choose the exchange (Form 8824); otherwise the sale price must include everything you received.",
    ),
    (
        "WARN_GAMBLING_LEFT_OUT",
        "{0}: a W-2G for a gambling business, a treaty or nonresident position, an installment prize sale or unreconciled records is left out of this estimate; enter a gambling business on Self-employment, or your tax may be higher.",
    ),
    (
        "WARN_GAMBLING_BUSINESS_LEFT_OUT",
        "{0}: a gambling activity not confirmed as a professional gambling business is left out of this estimate; enter casual gambling on the gambling page, or your tax may be higher.",
    ),
    (
        "WARN_BUSINESS_LEFT_OUT",
        "{0}: a business that is passive, has special treatment, or is an unconfirmed wholly foreign service business isn't calculated, so it is left out of this estimate; your tax may be higher, or lower if it had a loss.",
    ),
    (
        "WARN_BUSINESS_LOSS_LEFT_OUT",
        "{0}: this business loss needs actual at-risk method facts (Form 6198). Without those facts, this business and its loss are left out of this estimate; if the loss is allowed, your tax may be lower.",
    ),
    (
        "WARN_DE_MINIMIS_EXPENSE_LEFT_OUT",
        "{0}: purchases not confirmed as eligible for the de minimis election aren't deducted in this estimate; enter them under their ordinary expense or depreciation treatment, or your tax may be lower.",
    ),
    (
        "WARN_BUSINESS_TIPS_LEFT_OUT",
        "{0}: self-employed tips not confirmed as qualified tips get no qualified-tips deduction; if they qualify, your tax may be lower.",
    ),
    (
        "WARN_BUSINESS_OVERTIME_LEFT_OUT",
        "{0}: Form 1099 overtime not confirmed as FLSA-qualified overtime gets no qualified-overtime deduction; if it qualifies, your tax may be lower.",
    ),
    (
        "WARN_FUEL_CREDIT_LEFT_OUT",
        "{0}: a fuel tax credit whose purchase, tax, use and business records aren't confirmed complete is left out of this estimate; if they are, your tax may be lower.",
    ),
    (
        "WARN_FUEL_USE_LEFT_OUT",
        "Fuel not confirmed as qualifying (its use, no other claim or waiver, distinct gallons, export proof or the line's own conditions) is left out of the fuel tax credit; if it qualifies, your tax may be lower.",
    ),
    (
        "WARN_FUEL_RECOVERY_LEFT_OUT",
        "{0}: a fuel-tax credit or refund not filed or received in 2026 isn't income on this return, so it is left out of this estimate; if it was, your tax may be higher.",
    ),
    //
    // Answers and unknowns.
    (
        "WARN_PRIOR_CASH_CARRY_LEFT_OUT",
        "A prior cash charity carryover with a changed filing unit or incomplete source worksheets isn't supported, so it is left out of this estimate; your tax may be lower.",
    ),
    (
        "WARN_EMPLOYEE_STOCK_SALE_LEFT_OUT",
        "{0}: an employee stock sale whose compensation or employer reporting isn't confirmed isn't supported, so this sale is left out of this estimate; your tax may be higher.",
    ),
    (
        "EXPLAIN_FARM_CONSERVATION_AMT_DEDUCTION",
        "The section 175 AMT deduction uses the same legal gross-income cap with the parcel's separate retained AMT carry amounts.",
    ),
    (
        "EXPLAIN_FARM_CONSERVATION_AMT_CARRY",
        "The unused AMT section 175 cost remains separate from regular-tax carry and from an assessment's nine-year spread.",
    ),
    (
        "WARN_ESTATE_TRUST_PAYMENT_LEFT_OUT",
        "{0}: without a timely Form 1041-T the box 13 code A estimated-tax payment stays the estate's or trust's, so it is left out of your payments.",
    ),
    (
        "WARN_ESTATE_TRUST_NOL_LEFT_OUT",
        "{0}: a final-year estate or trust net operating loss whose fiduciary history isn't established isn't supported, so it is left out of this estimate; your tax may be lower.",
    ),
    (
        "WARN_DISASTER_MITIGATION_LEFT_OUT",
        "{0}: a hazard-mitigation payment that doesn't qualify for section 139(g) isn't supported, so it is left out of this estimate; your tax may be higher.",
    ),
    (
        "WARN_NONBUSINESS_PROPERTY_LEFT_OUT",
        "{0}: nonbusiness royalty or rental property with special treatment isn't supported, so it is left out of this estimate; your tax may be higher, or lower if it had a loss.",
    ),
    (
        "WARN_WORTHLESS_SECURITY_LEFT_OUT",
        "{0}: a worthless security with special treatment isn't supported, so its loss is left out of this estimate; your tax may be lower.",
    ),
    (
        "WARN_BAD_DEBT_LEFT_OUT",
        "{0}: a bad debt from business or investment work, a security or a purchased debt with other basis changes isn't supported, so its loss is left out of this estimate; your tax may be lower.",
    ),
    (
        "WARN_BAD_DEBT_NOT_DEDUCTED",
        "{0}: assumed a bad-debt fact you weren't sure about doesn't hold, so no loss is taken this year; if it does, your tax may be lower. You can amend within 7 years.",
    ),
    (
        "WARN_RETURN_OF_CAPITAL_LEFT_OUT",
        "{0}: a return of capital or liquidation with special treatment isn't supported, so it is left out of this estimate; your tax may be higher, or lower if it had a loss.",
    ),
    (
        "WARN_IRA_NEXT_YEAR_DEPOSIT_LEFT_OUT",
        "A 2027 IRA deposit not designated for 2026 belongs to 2027, so it is taken out of your 2026 contributions; your tax may be lower if it was designated for 2026.",
    ),
    (
        "WARN_SAVINGS_BONDS_LEFT_OUT",
        "{0}: savings bonds with interest reported on an earlier return aren't supported, so this record is left out and no interest is excluded; the interest stays taxable and your tax may be higher.",
    ),
    (
        "EXPLAIN_CLEAN_VEHICLE_OWN_BUSINESS_TOTAL",
        "Form 8936 line 6: business credit from your new clean vehicle schedules.",
    ),
    (
        "EXPLAIN_CLEAN_VEHICLE_K1_BUSINESS_TOTAL",
        "Form 8936 line 7: current new clean vehicle credits from partnership and S corporation Schedules K-1.",
    ),
    ("LINE_FORM_N_OWN_BUSINESS_PART", "Form {0} credit from your vehicles"),
    ("LINE_FORM_N_K1_BUSINESS_PART", "Form {0} credit from Schedules K-1"),
    (
        "EXPLAIN_CLEAN_VEHICLE_BUSINESS_TOTAL",
        "Form 8936 line 8: new clean vehicle business credit before the Form 3800 limit.",
    ),
    (
        "EXPLAIN_CLEAN_VEHICLE_BUSINESS_PART",
        "{0}: business-use new clean vehicle credit before the Form 3800 limit.",
    ),
    (
        "WARN_MORTGAGE_CREDIT_LEFT_OUT",
        "A Mortgage Credit Certificate or reissue that isn't confirmed isn't supported, so the 2026 mortgage interest credit is left out of this estimate; your tax may be lower.",
    ),
    (
        "WARN_TIP_FICA_CREDIT_LEFT_OUT",
        "{0}: an employer tip FICA credit whose service, payroll or carryback records aren't confirmed isn't supported, so it is left out of this estimate; your tax may be lower.",
    ),
    (
        "WARN_TIP_FICA_CARRYFORWARD_LEFT_OUT",
        "A 2025 employer tip FICA credit carryforward with an unsupported history is left out of this estimate; your tax may be lower.",
    ),
    (
        "WARN_WORK_OPPORTUNITY_CREDIT_LEFT_OUT",
        "{0}: a work opportunity credit whose certifications or wages aren't confirmed isn't supported, so it is left out of this estimate; your tax may be lower. If you completed Form 5884 yourself, enter its credit in More credits → General business credits, line 4b; enter the related wages after the credit reduction.",
    ),
    (
        "WARN_RETIREMENT_PLAN_CREDIT_LEFT_OUT",
        "{0}: a small employer retirement plan credit whose eligibility isn't confirmed isn't supported, so it is left out of this estimate; your tax may be lower. If you completed Form 8881 yourself, enter its credits in More credits → General business credits, lines 1j, 1dd or 1ee; enter the related expenses after the credit reduction.",
    ),
    (
        "WARN_SMALL_EMPLOYER_HEALTH_NOT_SHOP",
        "{0}: coverage not bought through a SHOP Marketplace earns no small employer health insurance credit (section 45R), so no credit is figured and the premiums stay deducted in full.",
    ),
    (
        "WARN_SMALL_EMPLOYER_HEALTH_CREDIT_LEFT_OUT",
        "{0}: a small employer health insurance credit whose eligibility isn't confirmed isn't supported, so it is left out of this estimate; your tax may be lower. If you completed Form 8941 yourself, enter its credit in More credits → General business credits, line 4h; enter the premiums after the credit reduction.",
    ),
    (
        "WARN_PAID_LEAVE_CREDIT_LEFT_OUT",
        "{0}: a paid family and medical leave credit whose written policy and eligibility aren't confirmed isn't supported, so it is left out of this estimate; your tax may be lower. If you completed Form 8994 yourself, enter its credit in More credits → General business credits, line 4j; enter the related wages or employee benefit premiums after the credit reduction.",
    ),
    (
        "WARN_NONCASH_GIFT_LEFT_OUT",
        "{0}: a gift relying on an appraisal or Form 8283 Section B that isn't confirmed isn't supported, so it is left out of this estimate; your tax may be lower.",
    ),
    (
        "WARN_PRIOR_HOUSEHOLD_CARRY_LEFT_OUT",
        "A 2025 household-item charity carryover with a changed filing unit or incomplete records isn't supported, so it is left out of this estimate; your tax may be lower.",
    ),
    (
        "WARN_NUA_SALE_LEFT_OUT",
        "{0}: a sale of employer stock whose NUA deferral or lot history isn't confirmed isn't supported, so this sale is left out of this estimate; your tax may be higher.",
    ),
    (
        "WARN_NUA_ESTATE_TAX_LEFT_OUT",
        "{0}: the section 691(c) deduction for estate tax on inherited NUA isn't calculated, so it is left out of this estimate; your tax may be lower.",
    ),
    (
        "WARN_SCORP_PASSIVE_HISTORY_LEFT_OUT",
        "{0}: an S corporation passive-loss history with special rules isn't supported, so its suspended losses are left out of this estimate; your tax may be lower.",
    ),
    (
        "WARN_SOCIAL_LUMP_SUM_ELECTION_LEFT_OUT",
        "Earlier-year records that don't cover every lump sum for the same filers don't support the lump-sum election, so all benefits are taxed under the regular 2026 method; your tax may be lower.",
    ),
    (
        "WARN_SECTION_179_SPOUSE_RECORDS_LEFT_OUT",
        "Without your spouse's section 179 purchase and allocation records, no section 179 deduction is taken on this separate return: your 2026 elections are depreciated normally and earlier section 179 carryforwards wait for 2027; your tax may be lower.",
    ),
    ("LINE_TAXABLE_RELIEF_PAYMENTS", "Taxable relief payments"),
    (
        "EXPLAIN_TAXABLE_RELIEF_PAYMENTS",
        "Wildfire or disaster relief payments not confirmed as excluded under section 139M or 139. Schedule 1 line 8z; not wages, self-employment earnings or net investment income.",
    ),
    (
        "TAX_NAVIGATION_PROMPT",
        "Income without a form, undocumented income, income with no tax form, cash income, side jobs, gig or app work, tips not reported to an employer, jury duty pay, a prize or hobby income belong to Income without a form when the person wants to add income no form shows. Existing business receipts and expenses stay on Self-employment; actual employer, monthly and payroll facts for unreported tips stay on Wages. Software depreciation, units of production, ACRS, pre-MACRS property and income forecast belong to the actual asset record under Self-employment, Farming, Rental property or Employee expenses. Form 4562 follows that activity. Find the offered tax page matching the person's request across all available sections, even when another section is currently open. Match the selected destination's documented scope, form numbers, Internal Revenue Code references and common names. Prefer the page where the person can enter or correct the requested facts over a generic overview, checklist, review or merely related page. A word mentioned to distinguish another destination does not make this page a match. For every offered page, recognize both prefixed and bare identifiers case-insensitively: Form 1098 or 1098, Form 1099-R or 1099-r, and Section 199A or 199a. Rank an exact documented identifier match ahead of broad topic matches; distinguish form suffixes such as 1098-T, 1098-E, 1098-C, 1098-VLI and 1099-QA rather than treating them as the base form. For an ambiguous subject return suitable alternatives; return no match if no offered destination fits. For Section 199A, 199a, Form 8995, 8995, Form 8995-A, 8995-A, qualified business income, QBI and QBI deduction, rank the offered Self-employment page first, unless the query names rental property, Schedule E, a partnership, S corporation or K-1: then rank that page first. A plain vehicle, car or auto query ranks Deductions first (personal car loan interest); a specific vehicle intent takes priority. An electric car's clean vehicle credit belongs to More credits, a car used for work to Self-employment, Farming or Rental property, donated clothes, goods or a car to Gifts of property and carryovers, car purchase sales tax and vehicle registration tax to Taxes you paid. Selling your house belongs to Investment sales, but a business asset, vehicle or rental sold, scrapped or condemned belongs to its own record under Self-employment, Rental property or Employee expenses. Digital assets received as pay belong to Other income, their sale to Investment sales. Incentive stock options belong to Investment sales, other alternative minimum tax items (Form 6251) to Other taxes. Form 8824 or a 1031 exchange without another qualifier belongs to Rental property. Rental income on a Schedule K-1, K-1 royalties and selling a partnership interest or S corporation shares belong to Partnerships, S corporations, estates and trusts; royalties you receive directly, to Rental property. Forms 1099-NEC and 1099-K belong to Self-employment unless you were the payer's employee (Wages, Form 8919). The S corporation that paid a W-2 is selected on Wages. Leased vehicles and Schedule C returns belong to Self-employment; solo 401(k), profit-sharing and defined-benefit plans to Self-employed health insurance and retirement plan. Investment credit (Form 3468), Form 4255 and recapture follow the property's page; solar panels, batteries or heat pumps at your home are a Form 5695 carryover on More credits. A K-1 credit statement belongs to Partnerships, S corporations, estates and trusts, K-1 and Schedule K-3 foreign taxes likewise; credits on their own source form, Form 1097-BTC and your own foreign tax credit to More credits. Form 3115 follows the activity: Self-employment first, Farming for farm or Schedule F, Rental property for rental or Schedule E; crop-share rent is Farming, not Rental property. TIPS interest, nominee components, savings-bond interest already reported, seller-financed mortgage interest and FBAR countries belong to Interest and dividends; a TIPS loss, bond-premium deductions and GST tax to Investment interest and other deductions. A child's income on the parent's return (Form 8814) belongs to Interest and dividends, Form 8615 to Your parent's return. Tuition and Form 1098-T belong to Education credits, 529 and Coverdell withdrawals (Form 1099-Q) to Prizes, scholarships, relief payments, account payouts and other income. A Roth conversion belongs to Retirement and Social Security for the Form 1099-R and to IRA contributions for the nondeductible part (Form 8606). A nanny, babysitter or daycare belongs to Child and dependent care for the care credit and to Other taxes for the household employer's Schedule H. A death in 2026 belongs to About you or About your spouse, a child who died to Children and people you support. Community property and spouses living apart belong to Your 2026 return; a registered domestic partner and a dual-status or part-year return to About you; innocent spouse relief, injured spouse allocation and the choice to treat a nonresident spouse as a resident to About your spouse.",
    ),
    ("ISSUE_BUSINESS_INCOME_AND_LOSSES", "Business income and losses"),
    ("PERSON_YOU", "You"),
    ("PERSON_SPOUSE", "Your spouse"),
    ("PERSON_NAME", "{0} {1}"),
    ("LABEL_ENTRY_OTHER_DOCUMENT", "Income document {0,number,0}"),
    (
        "EXPLAIN_HOME_SHORT_PROCEEDS",
        "Form 8949 Part I box C column (d) proceeds for the sale of your main home; column (h) goes to Schedule D line 3.",
    ),
    (
        "EXPLAIN_HOME_SHORT_PROCEEDS_WITH_CODES",
        "Form 8949 Part I box C column (d) proceeds for the sale of your main home, column (f) code {0}; column (h) goes to Schedule D line 3.",
    ),
    (
        "EXPLAIN_HOME_SHORT_BASIS",
        "Form 8949 Part I box C column (e) basis for the sale of your main home; column (h) goes to Schedule D line 3.",
    ),
    (
        "EXPLAIN_HOME_SHORT_BASIS_WITH_CODES",
        "Form 8949 Part I box C column (e) basis for the sale of your main home, column (f) code {0}; column (h) goes to Schedule D line 3.",
    ),
    (
        "EXPLAIN_HOME_SHORT_ADJUSTMENT",
        "Form 8949 Part I box C column (g) adjustment for the sale of your main home; column (h) goes to Schedule D line 3.",
    ),
    (
        "EXPLAIN_HOME_SHORT_ADJUSTMENT_WITH_CODES",
        "Form 8949 Part I box C column (g) adjustment for the sale of your main home, column (f) code {0}; column (h) goes to Schedule D line 3.",
    ),
    (
        "EXPLAIN_HOME_SHORT_GAIN",
        "Form 8949 Part I box C column (h) gain for the sale of your main home; column (h) goes to Schedule D line 3.",
    ),
    (
        "EXPLAIN_HOME_SHORT_GAIN_WITH_CODES",
        "Form 8949 Part I box C column (h) gain for the sale of your main home, column (f) code {0}; column (h) goes to Schedule D line 3.",
    ),
    (
        "EXPLAIN_HOME_LONG_PROCEEDS",
        "Form 8949 Part II box F column (d) proceeds for the sale of your main home; column (h) goes to Schedule D line 10.",
    ),
    (
        "EXPLAIN_HOME_LONG_PROCEEDS_WITH_CODES",
        "Form 8949 Part II box F column (d) proceeds for the sale of your main home, column (f) code {0}; column (h) goes to Schedule D line 10.",
    ),
    (
        "EXPLAIN_HOME_LONG_BASIS",
        "Form 8949 Part II box F column (e) basis for the sale of your main home; column (h) goes to Schedule D line 10.",
    ),
    (
        "EXPLAIN_HOME_LONG_BASIS_WITH_CODES",
        "Form 8949 Part II box F column (e) basis for the sale of your main home, column (f) code {0}; column (h) goes to Schedule D line 10.",
    ),
    (
        "EXPLAIN_HOME_LONG_ADJUSTMENT",
        "Form 8949 Part II box F column (g) adjustment for the sale of your main home; column (h) goes to Schedule D line 10.",
    ),
    (
        "EXPLAIN_HOME_LONG_ADJUSTMENT_WITH_CODES",
        "Form 8949 Part II box F column (g) adjustment for the sale of your main home, column (f) code {0}; column (h) goes to Schedule D line 10.",
    ),
    (
        "EXPLAIN_HOME_LONG_GAIN",
        "Form 8949 Part II box F column (h) gain for the sale of your main home; column (h) goes to Schedule D line 10.",
    ),
    (
        "EXPLAIN_HOME_LONG_GAIN_WITH_CODES",
        "Form 8949 Part II box F column (h) gain for the sale of your main home, column (f) code {0}; column (h) goes to Schedule D line 10.",
    ),
    (
        "EXPLAIN_CASUALTY_SHORT_NET",
        "Signed short- or long-term personal-use casualty result when Form 4684 has an overall net gain; entered once on Schedule D line 4, then coordinated with other capital gains and losses.",
    ),
    (
        "EXPLAIN_CASUALTY_LONG_NET",
        "Signed short- or long-term personal-use casualty result when Form 4684 has an overall net gain; entered once on Schedule D line 11, then coordinated with other capital gains and losses.",
    ),
    (
        "NOTE_PUBLIC_STOCK_CARRYOVER_RECORDS",
        "2025 public-stock carryover: retain the original donation on {0,date} of {1} to {2}, the public-market/basis records, 2025 return and carryover worksheet ({3}), and any original Form 8283 Section A. If the original noncash claim or similar-items group exceeds $500 before income limits, publicly traded securities use Form 8283 Section A, including in a carryover year. This original balance expires after 2030 if unused; the on-screen estimate does not prepare the attachment. Verify the 2026 floor against final IRS instructions before filing.",
    ),
    (
        "NOTE_HOUSEHOLD_CARRYOVER_RECORDS",
        "2025 household-furnishing carryover: retain the {0,date} donation of {1} to {2}, comparable-sales and condition evidence ({3}), and the filed 2025 Schedule A, Form 8283 Section A and carry worksheet ({4}). Attach the original completed Form 8283 to a 2026 return claiming this carryover. The original balance expires after 2030 if unused; this on-screen estimate does not prepare the attachment. Verify the 2026 floor against final IRS instructions before filing.",
    ),
    (
        "EXPLAIN_FORM8846_FOOD_HOURLY_BASIS",
        "2026 Form 8846 uses a {0,number,currency} hourly wage basis for food or beverage customer tips.",
    ),
    (
        "EXPLAIN_FORM8846_BEAUTY_HOURLY_BASIS",
        "2026 Form 8846 uses a {0,number,currency} hourly wage basis for qualifying beauty service customer tips.",
    ),
    (
        "EXPLAIN_FORM8846_UNRESOLVED_HOURLY_BASIS",
        "2026 Form 8846 uses a {0,number,currency} hourly wage basis for unresolved service customer tips.",
    ),
    (
        "EXPLAIN_BUSINESS_CREDIT_CURRENT_NONPASSIVE",
        "Form 3800 Part III line {0} column (e): {1,number,0} {2} credit.",
    ),
    (
        "EXPLAIN_BUSINESS_CREDIT_CURRENT_PASSIVE",
        "Form 3800 Part III line {0} column (d): {1,number,0} {2} credit.",
    ),
    (
        "EXPLAIN_BUSINESS_CREDIT_CARRY_NONPASSIVE",
        "Form 3800 Part IV line {0} column (f): {1,number,0} {2} credit.",
    ),
    (
        "EXPLAIN_BUSINESS_CREDIT_CARRY_PASSIVE",
        "Form 3800 Part IV line {0} column (d): {1,number,0} {2} credit.",
    ),
    ("EXPLAIN_FORM8379_WAGES_JOINT", "Income reported on Forms W-2, shown on the joint return."),
    (
        "EXPLAIN_FORM8379_WAGES_INJURED",
        "Income reported on Forms W-2, allocated to the injured spouse.",
    ),
    (
        "EXPLAIN_FORM8379_WAGES_OTHER",
        "Income reported on Forms W-2, allocated to the other spouse.",
    ),
    ("EXPLAIN_FORM8379_OTHER_INCOME_JOINT", "All other income, shown on the joint return."),
    ("EXPLAIN_FORM8379_OTHER_INCOME_INJURED", "All other income, allocated to the injured spouse."),
    ("EXPLAIN_FORM8379_OTHER_INCOME_OTHER", "All other income, allocated to the other spouse."),
    ("EXPLAIN_FORM8379_ADJUSTMENTS_JOINT", "Adjustments to income, shown on the joint return."),
    (
        "EXPLAIN_FORM8379_ADJUSTMENTS_INJURED",
        "Adjustments to income, allocated to the injured spouse.",
    ),
    ("EXPLAIN_FORM8379_ADJUSTMENTS_OTHER", "Adjustments to income, allocated to the other spouse."),
    (
        "EXPLAIN_FORM8379_DEDUCTIONS_JOINT",
        "Standard deduction or itemized deductions, shown on the joint return.",
    ),
    (
        "EXPLAIN_FORM8379_DEDUCTIONS_INJURED",
        "Standard deduction or itemized deductions, allocated to the injured spouse.",
    ),
    (
        "EXPLAIN_FORM8379_DEDUCTIONS_OTHER",
        "Standard deduction or itemized deductions, allocated to the other spouse.",
    ),
    ("EXPLAIN_FORM8379_NONREFUNDABLE_JOINT", "Nonrefundable credits, shown on the joint return."),
    (
        "EXPLAIN_FORM8379_NONREFUNDABLE_INJURED",
        "Nonrefundable credits, allocated to the injured spouse.",
    ),
    (
        "EXPLAIN_FORM8379_NONREFUNDABLE_OTHER",
        "Nonrefundable credits, allocated to the other spouse.",
    ),
    (
        "EXPLAIN_FORM8379_REFUNDABLE_JOINT",
        "Refundable credits (not the earned income credit), shown on the joint return.",
    ),
    (
        "EXPLAIN_FORM8379_REFUNDABLE_INJURED",
        "Refundable credits (not the earned income credit), allocated to the injured spouse.",
    ),
    (
        "EXPLAIN_FORM8379_REFUNDABLE_OTHER",
        "Refundable credits (not the earned income credit), allocated to the other spouse.",
    ),
    ("EXPLAIN_FORM8379_OTHER_TAX_JOINT", "Other taxes, shown on the joint return."),
    ("EXPLAIN_FORM8379_OTHER_TAX_INJURED", "Other taxes, allocated to the injured spouse."),
    ("EXPLAIN_FORM8379_OTHER_TAX_OTHER", "Other taxes, allocated to the other spouse."),
    (
        "EXPLAIN_FORM8379_WITHHOLDING_JOINT",
        "Federal income tax withheld, shown on the joint return.",
    ),
    (
        "EXPLAIN_FORM8379_WITHHOLDING_INJURED",
        "Federal income tax withheld, allocated to the injured spouse.",
    ),
    (
        "EXPLAIN_FORM8379_WITHHOLDING_OTHER",
        "Federal income tax withheld, allocated to the other spouse.",
    ),
    ("EXPLAIN_FORM8379_PAYMENTS_JOINT", "Payments, shown on the joint return."),
    ("EXPLAIN_FORM8379_PAYMENTS_INJURED", "Payments, allocated to the injured spouse."),
    ("EXPLAIN_FORM8379_PAYMENTS_OTHER", "Payments, allocated to the other spouse."),
    (
        "EXPLAIN_BUSINESS_N_FORM_8829_INDIRECT_SHARE",
        "Line 12 column (b) times the business percentage (line 7: {0} square feet of {1}, {2,number,0.00}%).",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM_8829_OPERATING_SHARE",
        "Line 23 column (b) times the business percentage ({0,number,0.00}%).",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM_8829_BUSINESS_BASIS",
        "Business basis of the building: line 39 times the business percentage ({0,number,0.00}%).",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM_8829_DEPRECIATION_ALLOWABLE",
        "Line 40 times line 41, {0,number,0.000}%: 39-year straight line, mid-month; the 2026 percentage for the first-use month, else 2.564%.",
    ),
    (
        "EXPLAIN_FORM_8815_EDUCATIONAL_BENEFITS",
        "Form 8815 line 3: tax-free scholarships, grants, refunds, employer or veterans' assistance and 529 or Coverdell payments received for those people.",
    ),
    (
        "EXPLAIN_FORM_8815_ELIGIBLE_INTEREST",
        "Form 8815 line 8: the interest on line 6 in the proportion of line 4 to line 5, or all of it when line 4 is at least line 5.",
    ),
    (
        "EXPLAIN_FORM_8815_INTEREST",
        "Form 8815 line 6 (Form 8818 line 8): the interest included in the proceeds.",
    ),
    (
        "EXPLAIN_FORM_8815_MODIFIED_AGI",
        "Form 8815 line 9: adjusted gross income before the savings bond exclusion and the student loan interest deduction, plus the foreign earned income and housing exclusions and the adoption benefits exclusion (section 135(c)(4)).",
    ),
    ("EXPLAIN_FORM_8815_NET_EXPENSES", "Form 8815 line 4: line 2 less line 3, not below zero."),
    (
        "EXPLAIN_FORM_8815_PHASEOUT_REDUCTION",
        "Form 8815 line 13: line 8 times the part of the phaseout range that line 9 is above the 2026 start of $101,800 ($152,650 on a joint return); the range is $15,000 ($30,000 joint).",
    ),
    (
        "EXPLAIN_FORM_8815_PROCEEDS",
        "Form 8815 line 5 (Form 8818 line 6): principal and interest from the series EE and I bonds issued after 1989 that you cashed in 2026.",
    ),
    (
        "EXPLAIN_FORM_8815_QUALIFIED_EXPENSES",
        "Form 8815 line 2: tuition and fees paid in 2026 for the people on the savings bond record, plus 529 plan and Coverdell contributions for them, after the expenses an education credit takes into account.",
    ),
    (
        "EXPLAIN_FORM_N_INJURED_SHARE_OF_TAX",
        "Form 1040 line 18 times the injured spouse's {0,number,0.00}% of both spouses' married filing separately tax ({1,number,currency} and {2,number,currency}).",
    ),
    ("ISSUE_INCOME_DOCUMENT", "Income document {0,number,0}"),
    ("REVIEW_RECORD_STATUS", "{0}: {1}"),
    ("REVIEW_RECORD_ITEM", "{0}: item {1,number,0}"),
    ("ISSUE_REPAYMENT_SOURCE_YEAR", "{0,number,0} repayment return — {1}"),
    ("PERSON_YOU_AND_SPOUSE", "You and your spouse"),
    ("ISSUE_SOURCE_MONTH", "{0} — month {1,number,0}"),
    ("ISSUE_PERSON_DATE", "{0} — {1,date}"),
    (
        "WARN_PARTICIPATION_LEFT_OUT",
        "{0} was left out because its participation scope or aggregation is not confirmed. Leaving out its income, losses, deductions and credits may make your tax higher or lower.",
    ),
    ("ISSUE_SECTION_1231", "Section 1231 property gains and losses"),
    ("SECTION_SETUP", "Your 2026 return"),
    ("SECTION_PEOPLE", "About you"),
    ("SECTION_SPOUSE", "About your spouse"),
    ("SECTION_CHILDREN", "Children and people you support"),
    ("SECTION_INCOME_BY_PERIOD", "Income by period"),
    ("REVIEW_TOLD_US", "What you told us"),
    (
        "EXPLAIN_8862_AGE_SPOUSE",
        "Your spouse’s age at the end of 2026 (at least 25 but under 65 for one filer to claim the credit).",
    ),
    (
        "EXPLAIN_8862_AGE_TAXPAYER",
        "Your age at the end of 2026 (at least 25 but under 65 for one filer to claim the credit).",
    ),
    (
        "EXPLAIN_8862_ANOTHER_QUALIFYING_CHILD_NO",
        "No: whether a filer could be another taxpayer's qualifying child for 2026.",
    ),
    (
        "EXPLAIN_8862_ANOTHER_QUALIFYING_CHILD_YES",
        "Yes: whether a filer could be another taxpayer's qualifying child for 2026.",
    ),
    ("EXPLAIN_8862_CHECKED_AOTC", "The American Opportunity Tax Credit box is checked."),
    (
        "EXPLAIN_8862_CHECKED_CHILD_CREDITS",
        "The Child Tax Credit/Additional Child Tax Credit/Credit for Other Dependents box is checked.",
    ),
    ("EXPLAIN_8862_CHECKED_EIC", "The Earned Income Credit box is checked."),
    ("EXPLAIN_8862_CHILD_HOME_DAYS", "Days {0} lived with you in the United States during 2026."),
    (
        "EXPLAIN_8862_CHILD_HOME_DAYS_EXCEPTION",
        "Days {0} lived with you in the United States during 2026 (365 under the newborn, death or temporary-absence rule).",
    ),
    (
        "EXPLAIN_8862_CLAIMABLE_SPOUSE_NO",
        "No: whether your spouse can be claimed as a dependent on another taxpayer's return.",
    ),
    (
        "EXPLAIN_8862_CLAIMABLE_SPOUSE_YES",
        "Yes: whether your spouse can be claimed as a dependent on another taxpayer's return.",
    ),
    (
        "EXPLAIN_8862_CLAIMABLE_TAXPAYER_NO",
        "No: whether you can be claimed as a dependent on another taxpayer's return.",
    ),
    (
        "EXPLAIN_8862_CLAIMABLE_TAXPAYER_YES",
        "Yes: whether you can be claimed as a dependent on another taxpayer's return.",
    ),
    (
        "EXPLAIN_8862_FINDING_NOT_ONLY_INCOME",
        "No: the earlier finding was not only misreported earned or investment income.",
    ),
    (
        "EXPLAIN_8862_FOUR_PRIOR_YEARS_NO",
        "No: whether the Hope or American opportunity credit was claimed for {0} in any 4 earlier tax years (part of the Form 8863 line 23 confirmation).",
    ),
    (
        "EXPLAIN_8862_FOUR_PRIOR_YEARS_YES",
        "Yes: whether the Hope or American opportunity credit was claimed for {0} in any 4 earlier tax years (part of the Form 8863 line 23 confirmation).",
    ),
    (
        "EXPLAIN_8862_INCOME_REPORTING_ONLY",
        "Yes: the only reason was incorrectly reported earned or investment income; the rest of Part II is skipped.",
    ),
    ("EXPLAIN_8862_LISTED_CHILD", "Child {0,number,0}: {1}."),
    ("EXPLAIN_8862_LISTED_OTHER_DEPENDENT", "Other dependent {0,number,0}: {1}."),
    (
        "EXPLAIN_8862_STUDENT_ELIGIBLE_NO",
        "No: {0} meets the eligible-student requirements for the American opportunity credit in 2026.",
    ),
    (
        "EXPLAIN_8862_STUDENT_ELIGIBLE_YES",
        "Yes: {0} meets the eligible-student requirements for the American opportunity credit in 2026.",
    ),
    (
        "EXPLAIN_8862_US_HOME_DAYS_SPOUSE",
        "Days during 2026 that your spouse’s main home was in the United States (at least 183 to claim the credit).",
    ),
    (
        "EXPLAIN_8862_US_HOME_DAYS_TAXPAYER",
        "Days during 2026 that your main home was in the United States (at least 183 to claim the credit).",
    ),
    (
        "EXPLAIN_RMD_LIFE_EXPECTANCY_CONTINUES",
        "Life-expectancy payments continue under the entered beneficiary status.",
    ),
    (
        "EXPLAIN_RMD_ACCOUNT_DEADLINE",
        "The remaining account must be emptied by December 31, {0,number,0}.",
    ),
    (
        "EXPLAIN_RMD_EXHAUSTED_LIFE_EXPECTANCY",
        "Life expectancy is exhausted; the full account must be distributed. The shortfall is the current year-end remaining value, including earnings, rather than last year's balance.",
    ),
    (
        "EXPLAIN_RMD_SPOUSE_UNIFORM_LIFETIME",
        "The sole surviving spouse retained inherited status and elected the Uniform Lifetime Table for the current year's age; the deceased owner's remaining expectancy also remains a floor when required.",
    ),
    (
        "EXPLAIN_RMD_SPOUSE_SINGLE_LIFE",
        "The surviving spouse's Single Life expectancy is recalculated at the current year's age, compared with the deceased owner's remaining expectancy when required.",
    ),
    (
        "EXPLAIN_RMD_ANNUAL_LIFE_EXPECTANCY",
        "Annual life-expectancy minimum applies; the initial factor is reduced for every elapsed year, including years covered by prior IRS penalty relief.",
    ),
    (
        "EXPLAIN_RMD_SPOUSE_MINIMUM_NOT_BEGUN",
        "The sole surviving spouse's inherited minimum has not begun: payments may be delayed until the later of the year after death or the year the original owner would reach the applicable RMD age.",
    ),
    (
        "EXPLAIN_RMD_NO_ANNUAL_MINIMUM",
        "No annual beneficiary minimum precedes the final payout deadline because the owner died before the required beginning date.",
    ),
    (
        "EXPLAIN_RMD_INHERITED_MINIMUM",
        "{0} {1} This minimum does not determine whether a withdrawal is taxable income or satisfy another account group's obligation.",
    ),
    ("CHECKLIST_FROM_YOUR_DOCUMENT", "{0}: from your {1}"),
    ("SUMMARY_TOPICS_NOT_CALCULATED", "{0} — not calculated yet"),
    ("TOPIC_ALTERNATIVE_MINIMUM_TAX_ITEMS", "Alternative minimum tax items"),
    ("TOPIC_ANOTHER_CREDIT_NOT_LISTED", "Another credit not listed"),
    ("OTHER_TAXES_TOO_MANY", "Enter at most 24 Other taxes items."),
    ("TOPIC_ANOTHER_TAX_NOT_LISTED", "Another tax not listed"),
    ("TOPIC_BUSINESS_CREDITS", "Business credits"),
    ("TOPIC_BUSINESS_OR_RENTAL_PROPERTY_SOLD", "Business or rental property sold"),
    (
        "TOPIC_CASUALTY_AND_THEFT_OF_BUSINESS_OR_INVESTMENT_PROPERTY",
        "Casualty and theft of business or investment property",
    ),
    ("TOPIC_CHILD_AND_DEPENDENT_CARE", "Child and dependent care"),
    ("TOPIC_CREDIT_RECAPTURE_AND_LOOK_BACK_INTEREST", "Credit recapture and look-back interest"),
    ("TOPIC_DEFERRED_SECTION_965_TAX", "Deferred section 965 tax"),
    ("TOPIC_EARNED_INCOME_CREDIT", "Earned income credit"),
    ("TOPIC_FARMING", "Farming"),
    ("TOPIC_FOREIGN_CORPORATIONS_AND_PFICS", "Foreign corporations and PFICs"),
    ("TOPIC_INVESTMENT_SALES", "Investment sales"),
    ("TOPIC_MINISTERS_AND_CHURCH_EMPLOYEES", "Ministers and church employees"),
    ("TOPIC_OTHER_ADJUSTMENTS", "Other adjustments"),
    ("TOPIC_OTHER_GIFTS_OF_PROPERTY", "Other gifts of property"),
    ("TOPIC_OTHER_ITEMIZED_DEDUCTIONS", "Other itemized deductions"),
    ("TOPIC_RETIREMENT_AND_SOCIAL_SECURITY", "Retirement and Social Security"),
    ("TOPIC_SELF_EMPLOYMENT", "Self-employment"),
    ("TOPIC_WAGES", "Wages"),
    ("TOPIC_TAX_TREATIES", "Tax treaties"),
    ("SECTION_REVIEW", "Review"),
    ("SECTION_DOCUMENTS", "Document center"),
    ("SECTION_INCOME_WITHOUT_FORM", "Income without a form"),
    ("LINE_INCOME_WITHOUT_FORM_OTHER", "Other income without a form"),
    (
        "ISSUE_WITHOUT_FORM_COMMUNITY_SOURCE_NEEDED",
        "To divide this income for a community-property return, its actual recipient and community or separate-property treatment are needed.",
    ),
    ("WITHOUT_FORM_SELF_WORK", "Work for yourself"),
    ("WITHOUT_FORM_TIPS", "Tips not reported to your employer"),
    ("WITHOUT_FORM_OTHER", "Other income: {0,number,currency}"),
    ("SECTION_CHECKLIST", "Beyond your documents"),
    ("SECTION_PAYMENTS", "Federal payments"),
    ("SECTION_DEDUCTIONS", "Deductions"),
    ("SECTION_OTHER_TAXES", "Other taxes"),
    ("SECTION_MORE_ADJUSTMENTS", "More adjustments"),
    ("SECTION_OFTEN_MISSED", "Situations people often miss"),
    ("SECTION_MORE_CREDITS", "More credits"),
    ("SECTION_MARKETPLACE", "Health insurance marketplace"),
    ("SECTION_SAVER", "Retirement savings credit"),
    ("SECTION_EDUCATION", "Education credits"),
    ("SECTION_OTHER_ITEMIZED", "Investment interest and other deductions"),
    ("SECTION_GIFT_CARRYOVERS", "Gifts of property and carryovers"),
    ("SECTION_MORTGAGE", "Home mortgage interest"),
    ("SECTION_TAXES", "Taxes you paid"),
    ("SECTION_MEDICAL", "Medical and dental expenses"),
    ("SECTION_STUDENT_LOAN", "Student loan interest"),
    ("SECTION_EDUCATOR", "Educator expenses"),
    ("SECTION_EMPLOYEE_EXPENSES", "Reservist, performing artist and fee-basis official expenses"),
    ("SECTION_IRA_CONTRIBUTIONS", "IRA contributions"),
    ("SECTION_SELF_EMPLOYED_BENEFITS", "Self-employed health insurance and retirement plan"),
    ("SECTION_HSA", "Health and medical savings accounts"),
    ("SECTION_LONG_TERM_CARE_BENEFITS", "Long-term care benefits"),
    ("SECTION_CANCELED_DEBT", "Canceled debt"),
    ("LINE_FOREIGN_PRIOR_YEAR_TAXPAYER_EXCLUSION", "Exclusion for 2025 services (you)"),
    ("LINE_FOREIGN_PRIOR_YEAR_SPOUSE_EXCLUSION", "Exclusion for 2025 services (your spouse)"),
    (
        "ISSUE_FOREIGN_AWARDS_ABOVE_PAY",
        "Direct prior-year pay and awards cannot exceed this person’s entered W-2 foreign pay and foreign-employer pay. Correct Work abroad or Wages.",
    ),
    (
        "EXPLAIN_FOREIGN_PRIOR_YEAR_EXCLUSION",
        "Form 2555 line 45: pay received in 2026 for qualifying 2025 services, limited by retained 2025 exclusion capacity. Direct pay and award portions are counted once.",
    ),
    ("SECTION_WORK_ABROAD", "Work abroad"),
    ("SECTION_PASS_THROUGH", "Partnerships, S corporations, estates and trusts"),
    ("SECTION_FARMING", "Farming"),
    ("SECTION_GAMBLING", "Gambling winnings"),
    (
        "SECTION_OTHER_INCOME",
        "Prizes, scholarships, relief payments, account payouts and other income",
    ),
    ("SECTION_GOVERNMENT_BENEFITS", "Unemployment and state refunds"),
    ("SECTION_RENTALS", "Rental property"),
    ("SECTION_SELF_EMPLOYMENT", "Self-employment"),
    ("SECTION_RETIREMENT", "Retirement and Social Security"),
    ("SECTION_INVESTMENT_SALES", "Investment sales"),
    ("SECTION_INTEREST_DIVIDENDS", "Interest and dividends"),
    ("SECTION_WAGES", "Wages"),
    ("SECTION_INJURED_SPOUSE", "Injured spouse allocation (Form 8379)"),
    ("SECTION_PARENT_TAX", "Your parent's return (Form 8615)"),
    ("SUMMARY_RETURN_PATH", "Your return: {0}"),
    (
        "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
        "Review the document amounts; they must be within the supported input range.",
    ),
    (
        "ISSUE_INTERVIEW_REVIEW_THE_ACCEPTED_PAGE_VALUES_BEFORE_65B6D7B7",
        "Review the accepted page values before saving.",
    ),
    (
        "ISSUE_INTERVIEW_THE_RECORD_IDENTITY_LIMIT_HAS_BEEN_78BC135C",
        "The record identity limit has been reached.",
    ),
    (
        "ISSUE_PARTNER_EXCESS_LOSS_QBI_AND_PROPERTY_CHARACTER",
        "Resolve partnership QBI and property character before excess-loss allocation.",
    ),
    (
        "ISSUE_PARTNERSHIP_RECONCILE_BOX_14_LESS_GUARANTEED_PAYMENTS_F10E8824",
        "Reconcile box 14 code A after guaranteed payments: a limited partner includes only service payments, trading earnings need separate treatment, and an ordinary loss held back by basis, at-risk or passive limits needs a compatible self-employment share.",
    ),
    (
        "ISSUE_QBI_PTP_COMPONENT_CLASSIFICATION",
        "Separately reported PTP items need their actual business and specified-service source classification before the income limit can be applied.",
    ),
    (
        "ISSUE_PARTNERSHIP_QBI_PROVIDE_PARTNERSHIP_GROSS_SE_INCOME_FOR_45332F9E",
        "Provide partnership gross SE income for the owner-level QBI deduction allocation; net earnings cannot substitute for gross income.",
    ),
    (
        "ISSUE_PARTNERSHIP_QBI_RECONCILE_PARTNERSHIP_QBI_WITH_THE_RESOLVED_708814BB",
        "Reconcile partnership QBI with the resolved passive limit and partner deductions.",
    ),
    (
        "ISSUE_SCORP_RESOLVE_TRADING_BUSINESS_CLASSIFICATION_AND_MATERIAL_PARTICIPA_EDFB91CC",
        "Resolve trading-business classification and material participation for the investment-interest income limit.",
    ),
    (
        "ISSUE_SCORP_TRADING_INCOME_WITH_RELEASED_PRIOR_PASSIVE_8CD8F8E2",
        "Trading income with released prior passive losses needs separate investment-interest expense allocation.",
    ),
    (
        "ISSUE_SECTION1231_ENTER_INDEPENDENT_AMT_K_1_RECORDS_FOR_A9D91A04",
        "Enter independent AMT K-1 records for the section 1231 worksheet.",
    ),
    (
        "ISSUE_SECTION1231_RESOLVE_DIRECT_FARM_LAND_BASIS__HOLDING_PERIOD_E4702532",
        "Resolve direct farm-land basis, holding period and transaction eligibility before section 1231 netting.",
    ),
    (
        "ISSUE_SECTION1231_THE_PROPERTY_QBI_RESULT_HAS_NO_F4545055",
        "The property QBI result has no matching business source.",
    ),
    (
        "NOTE_ESTATE_TRUST_QBI",
        "Estate/trust document {0,number,integer}, box 14 code I trade or business QBI after SSTB eligibility: {1,number,currency}; allocated W-2 wages {2,number,currency}; qualified-property basis {3,number,currency}. Enters the shared QBI deduction engine with the boxes 6–8 income already on Schedule E; other return limits remain separate.",
    ),
    (
        "LINE_MARKETPLACE_ITERATIVE_CREDIT_WORK",
        "Iterative premium credit, selected Step 4 work amount",
    ),
    (
        "LINE_MARKETPLACE_ITERATIVE_DEDUCTION_WORK",
        "Iterative specified-premium worksheet component, selected Step 5 work amount",
    ),
    (
        "EXPLAIN_MARKETPLACE_ITERATIVE_PAIR",
        "Publication 974 accepts the Step 4 premium credit together with the Step 5 specified-premium worksheet component when both changes are strictly below $1. Combine a signed component with the selected nonspecified premiums before claiming the nonnegative health deduction. These cent work amounts remain separate from rounded form rows and allocated to their existing policies and businesses.",
    ),
    ("MESSAGE_PERSON_NO_LONGER_ON_RETURN", "{0} is no longer on this return. Remove their row."),
    (
        "ISSUE_SAVINGS_BOND_INTEREST_ABOVE_1099_INT_BOX_3",
        "The interest on the savings bond record is more than the box 3 interest of the 1099-INT statements entered. Enter the 1099-INT for the bonds you cashed, or correct the interest.",
    ),
    (
        "EDUCATION_INSURANCE_REFUND_ABOVE_BOX_10",
        "Reconcile the current-expense insurance allocation with the accepted Form 1098-T box 10 amounts and the other refunds.",
    ),
    (
        "ISSUE_EDUCATION_RECAPTURE_COMPLEX_PRIOR_RETURN",
        "An earlier education credit needs its actual expense year and complete prior-return history established before calculating recapture.",
    ),
    ("ISSUE_PENSION_IRA_SPOUSE_COVERAGE", "Complete both IRA coverage records for a joint return."),
    (
        "ISSUE_PENSION_ROTH_COORDINATION",
        "Section 501(c)(18)(D) contributions with this employee's Roth IRA contributions or 529-to-Roth rollover require further verified contribution-limit coordination.",
    ),
    (
        "ISSUE_PENSION_IRA_PHASEOUT_COORDINATION",
        "Section 501(c)(18)(D) contributions combined with a partially phased-out traditional IRA deduction need verified limit-ordering coordination.",
    ),
    (
        "ISSUE_IRA_QTP_ROLLOVERS",
        "Confirm each 2026 direct 529-to-Roth transfer, its distinct source-account history and five-year seasoned amount after earlier transfers, its source contribution/earnings split, and the $35,000 beneficiary lifetime room across all accounts.",
    ),
    (
        "ISSUE_IRA_SPOUSE_SITUATION",
        "Choose the IRA spouse situation matching filing status and whether you lived together during the year.",
    ),
    (
        "ISSUE_SCORP_NOTE_ADVANCES_WITH_DATED_REPAYMENTS",
        "Dated note repayments with new principal advances require transaction-level basis and holding-period allocation.",
    ),
    (
        "ISSUE_YOUNG_PARTNER_HISTORICAL_LOSSES",
        "Young-partner historical losses require separate earned-income coordination.",
    ),
    (
        "ISSUE_PAPER_NOTHING_ENTERED",
        "Nothing is entered from this paper: enter it from the printed form, or delete it.",
    ),
    (
        "ISSUE_BBA_AUDIT_SCOPE",
        "Confirm one BBA-audit portfolio-interest adjustment, a simple ordinary-tax 2025 return, and no other tax-attribute effects. Other Form 8978 cases need a complete prior-year refigure.",
    ),
    (
        "ISSUE_RENTAL_HOME_FIXED_EXPENSES_EXCEED_RENT",
        "Mortgage interest, real estate taxes and rental-only expenses exceed the rent from a home used for personal purposes; this needs the Worksheet 5-1 line 2 and Schedule A coordination before an estimate.",
    ),
    (
        "ISSUE_RENTAL_BUILDING_DEPRECIATION_RECORDS",
        "Review building cost, land allocation and service date; this path uses post-1986 residential or nonresidential GDS.",
    ),
    (
        "ISSUE_RENTAL_HOME_PRIOR_PASSIVE_LOSSES",
        "Prior suspended passive losses for a property used as a home this year need their own allocation before an estimate.",
    ),
    (
        "ISSUE_8379_COMMUNITY_FEDERAL_TAX_DEBT",
        "Form 8379 line 5: a federal tax debt in a community property state is offset under that state's law (Rev. Rul. 2004-71 through 2004-74); the App does not apply it.",
    ),
    (
        "ISSUE_8379_COMMUNITY_SEPARATE_PROPERTY",
        "Form 8379 line 5: separate property items under a state's community property law need the state's allocation (Pub. 555); the App does not apply it.",
    ),
    (
        "ISSUE_8379_FARMLAND_INSTALLMENT_OFFSET",
        "Form 8379 line 20: the Form 1062 farmland installment offset on Form 1040 line 31 is not allocated between spouses by the App.",
    ),
    (
        "ISSUE_8379_WITHHOLDING_UNATTRIBUTABLE",
        "Form 8379 line 19: the federal income tax withheld cannot be attributed between you.",
    ),
    (
        "NOTE_INJURED_HALF_EDUCATOR",
        "Form 8379: {0,number,currency} for educator expenses (line 14) was divided equally because the return does not identify a spouse; the instructions allow you to choose an allocation.",
    ),
    (
        "NOTE_INJURED_HALF_IRA",
        "Form 8379: {0,number,currency} for the IRA deduction (line 14) was divided equally because the return does not identify a spouse; the instructions allow you to choose an allocation.",
    ),
    (
        "NOTE_INJURED_HALF_STUDENT_LOAN",
        "Form 8379: {0,number,currency} for student loan interest (line 14) was divided equally because the return does not identify a spouse; the instructions allow you to choose an allocation.",
    ),
    (
        "NOTE_INJURED_HALF_ADJUSTMENT",
        "Form 8379: {0,number,currency} for an adjustment (line 14) was divided equally because the return does not identify a spouse; the instructions allow you to choose an allocation.",
    ),
    (
        "NOTE_INJURED_HALF_OTHER_ADJUSTMENTS",
        "Form 8379: {0,number,currency} for other adjustments (line 14) was divided equally because the return does not identify a spouse; the instructions allow you to choose an allocation.",
    ),
    (
        "NOTE_INJURED_HALF_ITEMIZED",
        "Form 8379: {0,number,currency} for itemized deductions (line 15) was divided equally because the return does not identify a spouse; the instructions allow you to choose an allocation.",
    ),
    (
        "NOTE_INJURED_HALF_CHARITY",
        "Form 8379: {0,number,currency} for the non-itemizer charitable deduction (line 15) was divided equally because the return does not identify a spouse; the instructions allow you to choose an allocation.",
    ),
    (
        "NOTE_INJURED_HALF_VEHICLE_INTEREST",
        "Form 8379: {0,number,currency} for vehicle loan interest (line 15) was divided equally because the return does not identify a spouse; the instructions allow you to choose an allocation.",
    ),
    (
        "NOTE_INJURED_HALF_SCHEDULE_1A_QBI",
        "Form 8379: {0} for a Schedule 1-A or QBI deduction (line 15) was divided equally because the return does not identify a spouse; the instructions allow you to choose an allocation.",
    ),
    (
        "NOTE_INJURED_HALF_NONREFUNDABLE",
        "Form 8379: {0,number,currency} for other nonrefundable credits (line 16) was divided equally because the return does not identify a spouse; the instructions allow you to choose an allocation.",
    ),
    (
        "NOTE_INJURED_HALF_REFUNDABLE",
        "Form 8379: {0,number,currency} for other refundable credits (line 17) was divided equally because the return does not identify a spouse; the instructions allow you to choose an allocation.",
    ),
    (
        "NOTE_INJURED_HALF_OTHER_TAX",
        "Form 8379: {0,number,currency} for an other tax (line 18) was divided equally because the return does not identify a spouse; the instructions allow you to choose an allocation.",
    ),
    (
        "NOTE_INJURED_HALF_OTHER_TAXES",
        "Form 8379: {0,number,currency} for other taxes (line 18) was divided equally because the return does not identify a spouse; the instructions allow you to choose an allocation.",
    ),
    (
        "NOTE_SCORP_STOCK_BASIS",
        "S-corporation document {0}, stock basis: {1,number,currency}. Ending stock basis for the confirmed ordinary-business worksheet; separate AMT and other basis adjustments remain outside this worksheet.",
    ),
    (
        "NOTE_SCORP_LOAN_BASIS",
        "S-corporation document {0}, loan {2} basis: {1,number,currency}. Ending basis of this debt, kept separately from stock and other loans.",
    ),
    (
        "NOTE_SCORP_LOAN_RESTORATION",
        "S-corporation document {0}, loan {2} restoration: {1,number,currency}. Income used to restore this previously reduced debt basis.",
    ),
    ("USER_RETURNS", "Saved 2026 federal prototype return of {0}"),
    ("STORAGE_FORMAT", "Stored data format of {0}"),
    ("TEXT", "{0}"),
    ("DISPLAY_PREVIOUS_STEP", "Previous step"),
    ("DISPLAY_SAVE_AND_CONTINUE", "Save and continue"),
    ("DISPLAY_CLOSE_EDITOR", "Close editor"),
    ("DISPLAY_FINISH_REVIEW", "Finish review"),
    ("DISPLAY_RETURN_TO_REVIEW", "Return to review"),
    ("DISPLAY_COSTS", "Costs"),
    ("DISPLAY_REPAY_INCOME", "Income"),
    ("DISPLAY_SALE_AMOUNTS", "Amounts"),
    ("DISPLAY_DOCUMENT_DETAILS", "Form details"),
    ("DISPLAY_STUDENT", "Student"),
    ("DISPLAY_CARRY_GIFT", "Original gift"),
    ("DISPLAY_CARRY_BALANCE", "Carryover"),
    ("DISPLAY_PAYROLL_QBI", "Payroll and QBI"),
    ("DISPLAY_LIMITS_SPECIAL", "Limits and special cases"),
    ("DISPLAY_ASSET_PROPERTY", "Property"),
    ("DISPLAY_ASSET_HISTORY", "Depreciation and disposal"),
    ("DISPLAY_ASSET_INVESTMENT_CREDIT", "Investment credit"),
    ("DISPLAY_K1_1041_BOX14", "Box 14 and special"),
    ("DISPLAY_OTHER_BOXES", "Other boxes"),
    ("DISPLAY_K1S_DEDUCTIONS", "Boxes 11–13"),
    ("DISPLAY_K1S_EXEMPT", "Boxes 15–16"),
    ("DISPLAY_K1P_FUEL", "Box 20 code C fuel"),
    ("DISPLAY_BASIS", "Basis"),
    ("DISPLAY_CARRYOVER_REGULAR", "Regular tax"),
    ("DISPLAY_CARRYOVER_AMT", "Alternative minimum tax"),
    ("DISPLAY_LUMP_SUM_BENEFITS", "Benefits"),
    ("DISPLAY_RESULT_ESTIMATE", "Estimate"),
    ("DISPLAY_RESULT_RETURN", "Return"),
    ("DISPLAY_RESULT_NOTES", "Notes"),
    ("DISPLAY_RETURN_LINE", "Return line"),
    ("DISPLAY_HOW_CALCULATED", "How calculated"),
    ("DISPLAY_SALE_1099B", "1099-B sale"),
    ("DISPLAY_STOCK_PLAN_RECORD", "Stock plan record"),
    ("DISPLAY_SALE_FORM_3922", "Form 3922"),
    ("DISPLAY_MISC_INCOME", "Rents, royalties and other income"),
    ("DISPLAY_MISC_BUSINESS", "Business boxes"),
    (
        "NOTE_BOTH_2025_NONCASH_CARRYOVERS_MUST_RECONCILE_TO_THE_SAME_FILED_20_5A4E2EFA",
        "Both 2025 noncash carryovers must reconcile to the same filed 2025 charity-limit worksheet: the household gift retains its 50% class and the appreciated public stock retains its 30% class. Attach completed copies of the prior-year Form 8283 information separately for each carried gift on the 2026 return; neither original gift is another 2026 contribution. The five-year expiry of each balance remains after 2030.",
    ),
    (
        "NOTE_RETAIN_EACH_ORIGINAL_AND_INTERVENING_CHARITABLE_CARRYOVER_WORKSH_1293A333",
        "Retain each original and intervening charitable-carryover worksheet. The 2026 floor treatment follows enacted section 170; verify it against final 2026 IRS instructions before filing.",
    ),
    (
        "NOTE_8862_AOTC_BANNED",
        "Form 8862: the American opportunity tax credit is not allowed for 2026 because the {0}-year ban from the {1,number,0} disallowance runs through tax year {2,number,0}, so this estimate leaves it out. To appeal the ban you would claim the credit on a mailed return with Form 8862 attached; the IRS rejects an e-filed claim during the ban.",
    ),
    (
        "NOTE_8862_AOTC_BANNED_LIFETIME_LEARNING_USED",
        "{0}: the American opportunity credit is banned, so this student's expenses are figured for the lifetime learning credit, which the ban does not cover.",
    ),
    (
        "EXPLAIN_K1_RENTAL_SECTION1231_SUSPENDED",
        "The rental activity's section 1231 loss not allowed by the passive activity rules carries separately from its ordinary rental loss. Keep this amount with this activity for next year.",
    ),
    (
        "LINE_FORM8582_PART_IX_N_RENTAL_REAL_ESTATE_ORDINARY_ALLOWED",
        "K-1 {0} rental real estate: ordinary loss, allowed",
    ),
    (
        "LINE_FORM8582_PART_IX_N_RENTAL_REAL_ESTATE_ORDINARY_UNALLOWED",
        "K-1 {0} rental real estate: ordinary loss, unallowed",
    ),
    (
        "LINE_FORM8582_PART_IX_N_RENTAL_REAL_ESTATE_SECTION1231_ALLOWED",
        "K-1 {0} rental real estate: section 1231 loss, allowed",
    ),
    (
        "LINE_FORM8582_PART_IX_N_RENTAL_REAL_ESTATE_SECTION1231_UNALLOWED",
        "K-1 {0} rental real estate: section 1231 loss, unallowed",
    ),
    (
        "LINE_SCORP_N_RENTAL_REAL_ESTATE_SECTION1231_SUSPENDED",
        "S corporation {0} rental real estate: section 1231 loss carried forward",
    ),
    (
        "LINE_PARTNERSHIP_N_RENTAL_REAL_ESTATE_SECTION1231_SUSPENDED",
        "Partnership {0} rental real estate: section 1231 loss carried forward",
    ),
    (
        "LINE_AMT_SCORP_N_RENTAL_REAL_ESTATE_SECTION1231_SUSPENDED",
        "AMT S corporation {0} rental real estate: section 1231 loss carried forward",
    ),
    (
        "LINE_AMT_PARTNERSHIP_N_RENTAL_REAL_ESTATE_SECTION1231_SUSPENDED",
        "AMT Partnership {0} rental real estate: section 1231 loss carried forward",
    ),
    (
        "LINE_FORM8582_PART_IX_N_OTHER_RENTAL_ORDINARY_ALLOWED",
        "K-1 {0} other rental: ordinary loss, allowed",
    ),
    (
        "LINE_FORM8582_PART_IX_N_OTHER_RENTAL_ORDINARY_UNALLOWED",
        "K-1 {0} other rental: ordinary loss, unallowed",
    ),
    (
        "LINE_FORM8582_PART_IX_N_OTHER_RENTAL_SECTION1231_ALLOWED",
        "K-1 {0} other rental: section 1231 loss, allowed",
    ),
    (
        "LINE_FORM8582_PART_IX_N_OTHER_RENTAL_SECTION1231_UNALLOWED",
        "K-1 {0} other rental: section 1231 loss, unallowed",
    ),
    (
        "LINE_SCORP_N_OTHER_RENTAL_SECTION1231_SUSPENDED",
        "S corporation {0} other rental: section 1231 loss carried forward",
    ),
    (
        "LINE_PARTNERSHIP_N_OTHER_RENTAL_SECTION1231_SUSPENDED",
        "Partnership {0} other rental: section 1231 loss carried forward",
    ),
    (
        "LINE_AMT_SCORP_N_OTHER_RENTAL_SECTION1231_SUSPENDED",
        "AMT S corporation {0} other rental: section 1231 loss carried forward",
    ),
    (
        "LINE_AMT_PARTNERSHIP_N_OTHER_RENTAL_SECTION1231_SUSPENDED",
        "AMT Partnership {0} other rental: section 1231 loss carried forward",
    ),
    (
        "ISSUE_K1_RENTAL_PTP_SOURCE",
        "The rental activities of this publicly traded partnership need the same selected regular or AMT passive-loss source, including its checked section 179 allocation.",
    ),
    (
        "SITUATION_NAME_NONCASH_GIFTS",
        "Gifts of property needing a qualified appraisal (Form 8283 Section B), art, real estate, partial interests and conservation easements, inventory or business property, inherited, gifted, restricted or closely held property, the capital gain property basis election, and gifts with goods or services received in return (Form 8283, Form 1098-C, Schedule A lines 11 and 12)",
    ),
    (
        "ISSUE_SITUATION_NONCASH_GIFTS",
        "Gifts of property needing a qualified appraisal (Form 8283 Section B), art, real estate, partial interests and conservation easements, inventory or business property, inherited, gifted, restricted or closely held property, the capital gain property basis election, and gifts with goods or services received in return (Form 8283, Form 1098-C, Schedule A lines 11 and 12) is not calculated yet.",
    ),
    (
        "SITUATION_NAME_BUSINESS_CREDITS",
        "General business credits other than Forms 5884, 8846, 8881, 8941, 8994 and a K-1's low-income housing credit; a building you own directly (Forms 8609-A and 8586) is not calculated (Form 3800 and its source credits, including a fuel cell vehicle bought before 2022 and first placed in service in 2026, Schedule 3 line 6a)",
    ),
    (
        "ISSUE_SITUATION_BUSINESS_CREDITS",
        "Enter a completed source-form credit under More credits → General business credits (Form 3800), choosing the line the source form names. The App does not compute every source form from its underlying facts.",
    ),
    (
        "SITUATION_NAME_ELECTIVE_PAYMENT",
        "Schedule 3 line 13c (net elective payment election, Form 3800)",
    ),
    (
        "ISSUE_SITUATION_ELECTIVE_PAYMENT",
        "Eligible elective payment is entered with its source-form credit under More credits → General business credits (Form 3800); the net payment reaches Schedule 3 line 13c.",
    ),
    (
        "SITUATION_NAME_OTHER_CREDIT",
        "Other credits (Schedule 3 line 6z), including a fuel cell vehicle bought before 2022 and first placed in service in 2026",
    ),
    (
        "ISSUE_SITUATION_OTHER_CREDIT",
        "Other credits (Schedule 3 line 6z), including a fuel cell vehicle bought before 2022 and first placed in service in 2026 is not calculated yet.",
    ),
    (
        "SITUATION_NAME_OTHER_ADDITIONAL_TAXES",
        "Schedule 2 lines 1d–1f and 1y, 7, 8, 10, 12, 13a, 13g–13m and 19a–19c",
    ),
    (
        "ISSUE_SITUATION_OTHER_ADDITIONAL_TAXES",
        "Schedule 2 lines 1d–1f and 1y, 7, 8, 10, 12, 13a, 13g–13m and 19a–19c is not calculated yet.",
    ),
    (
        "ISSUE_MARKETPLACE_MARRIAGE_YEAR",
        "The alternative calculation for the year of marriage (Form 8962 Part V, lines 35 and 36, Pub. 974 Worksheets I and V) is not calculated.",
    ),
    (
        "ISSUE_MARKETPLACE_QSEHRA",
        "A QSEHRA permitted benefit (Form 8962 lines 11 to 23 column (e), Pub. 974 Worksheets N and Q) is not calculated.",
    ),
    (
        "ISSUE_MARKETPLACE_NOT_LAWFULLY_PRESENT",
        "Coverage of a person not lawfully present in the United States (Form 8962 lines 1 to 5 and column (f), Pub. 974 Worksheets A and B) is not calculated.",
    ),
    (
        "ISSUE_MARKETPLACE_THREE_TAX_FAMILIES",
        "A policy shared among three or more tax families (Form 8962 Part IV lines 30 to 33, Pub. 974 Allocation of Policy Amounts Among Three or More Taxpayers) is not calculated.",
    ),
    (
        "ISSUE_MARKETPLACE_UNPAID_PREMIUMS",
        "Enrollment premiums not paid by April 15, 2027 are left out of Form 8962 lines 11 to 23 column (a) and change the months that can take the credit; this is not calculated.",
    ),
    (
        "ISSUE_W2_BOX12_R_UNSUPPORTED",
        "W-2 box 12 code R (employer Archer MSA contributions, Form 8853) needs the Archer MSA topic, not yet supported from this page.",
    ),
    (
        "ISSUE_W2_BOX12_TA_UNSUPPORTED",
        "W-2 box 12 code TA (Trump account employer contribution) needs the Other situations list, not yet supported.",
    ),
    (
        "ISSUE_FARMER_GROSS_AMOUNT_UNAVAILABLE",
        "The farmer or fisher gross-income sources exceed the supported monetary calculation capacity. The qualification cannot be determined from a net amount or a truncated total. Review the source amounts; an independently established qualifying prior-year or next-year answer remains usable for its own payment calculation.",
    ),
    (
        "ISSUE_FARMER_GROSS_SOURCE_UNAVAILABLE",
        "The current farmer or fisher gross-income test needs complete source-owner gross statements and the actual farming or fishing character of recognized gains before other sources' losses. K-1 net income, Schedule K-3 foreign-tax gross, or an earlier sale's business-property label cannot replace those facts. A qualifying prior-year farmer or fisher answer independently supports the current penalty; a qualifying expected next-year answer independently supports next year's estimate.",
    ),
    (
        "ISSUE_2210_ESTIMATED_PAYMENT_DATES",
        "Form 2210 line 11 and Form 2210-F line 12 need the date of each estimated payment; enter the payments by date.",
    ),
    (
        "ISSUE_EMPLOYEE_TIPS_RECORDS",
        "Check the 2026 tip records: enter each month once, with reported tips no more than the tips received.",
    ),
    (
        "ISSUE_EMPLOYEE_WAGES_RECORDS",
        "Enter a valid Form 8919 reason and pay amounts; reason H needs the Form 1099 you received.",
    ),
    (
        "ISSUE_HOME_OFFICE_MORTGAGE_RECORD_MATCH",
        "When you itemize, enter the Form 1098 for the home with the office under Home mortgage interest as well; its interest or mortgage insurance premiums are less than the office's (Form 8829 line 10, Schedule A lines 8a and 8d).",
    ),
    (
        "ISSUE_HOME_OFFICE_PROPERTY_TAX_MATCH",
        "When you itemize, include the real estate taxes of the home with the office under Taxes you paid as well; they are less than the office's real estate taxes (Form 8829 Line 11 Worksheet, Schedule A line 5b).",
    ),
    (
        "ISSUE_HOME_OFFICE_MORTGAGE_LIMIT",
        "Home mortgage interest above the Pub 936 limit with an actual-expense home office is not calculated (Form 8829 lines 10 and 16).",
    ),
    (
        "ISSUE_HOME_OFFICE_MORTGAGE_INSURANCE_LIMIT",
        "Mortgage insurance premiums reduced by the adjusted gross income limit with an actual-expense home office need the Mortgage Insurance Premiums Deduction Worksheet for Form 8829 and are not calculated (Form 8829 lines 10 and 16).",
    ),
    (
        "ISSUE_HOME_OFFICE_MORTGAGE_CREDIT",
        "A mortgage interest credit with an actual-expense home office is not calculated (Form 8396 and Form 8829 line 10).",
    ),
    (
        "ISSUE_HOME_OFFICE_ESTATE_TAXES",
        "An estate's or trust's final-year state and local taxes with an actual-expense home office are not calculated (Form 8829 Line 11 Worksheet lines 1-4).",
    ),
    ("NOTE_WITH_LABEL", "{0}: {1}"),
    (
        "ISSUE_SECTION179_SELECTED_SOURCE_UNAVAILABLE",
        "Section 179 requires complete, consistent regular and AMT income, asset, carryover and before-deduction at-risk records; the selected source amount is unavailable.",
    ),
    (
        "ISSUE_ALLOCATE_AT_MOST_300_SQUARE_FEET_PER_MONTH_ACROSS_THIS_OWNER_S_SIMP_7581D3E9",
        "Allocate at most 300 square feet per month across this owner's simplified-method businesses; do not reuse the same space.",
    ),
    (
        "ISSUE_AN_ACTUAL_EXPENSE_HOME_OFFICE_FOR_A_FARM_OR_A_PROFESSIONAL_GAMBLING_FCF06994",
        "An actual-expense home office for a farm or a professional gambling business uses the Pub 587 worksheet instead of Form 8829 and is not calculated.",
    ),
    (
        "ISSUE_AN_HSA_ROLLOVER_IRA_TO_HSA_FUNDING_TRANSFER_RETURNED_EXCESS_CONTRIB_156458F7",
        "An HSA rollover, IRA-to-HSA funding transfer, returned excess contribution, coverage-loss test from 2025 or prohibited transaction (Form 8889 lines 10, 14b, 18 and 19; Form 5329 Part VII) is not calculated from the HSA page yet.",
    ),
    (
        "ISSUE_AN_OWNER_RECEIVING_A_SEP_CONTRIBUTION_IS_COVERED_BY_A_WORKPLACE_RET_D99C81A9",
        "An owner receiving a SEP contribution is covered by a workplace retirement plan. Update the IRA coverage answer.",
    ),
    (
        "ISSUE_A_2025_HOUSEHOLD_FURNISHING_CARRYOVER_ALONGSIDE_A_2026_REGULAR_OR_A_8534BA3B",
        "A 2025 household-furnishing carryover alongside a 2026 regular or AMT operating loss requires coordinated Form 172 and charitable-carryover worksheets, including on the standard route.",
    ),
    (
        "ISSUE_A_2025_HOUSEHOLD_FURNISHING_CARRYOVER_WITH_A_PRIOR_CASH_OR_OTHER_UN_2B274F41",
        "A 2025 household-furnishing carryover with a prior cash or other unsupported carryover class or a 2026 hosted-student/whaling 30%-class contribution needs a coordinated worksheet. One documented 2025 public-stock 30%-class carryover may be entered with this balance when the original-year combined worksheet is reconciled.",
    ),
    (
        "ISSUE_A_2025_HOUSEHOLD_FURNISHING_CARRYOVER_WITH_REGULAR_OR_AMT_OPERATING_A808853C",
        "A 2025 household-furnishing carryover with regular or AMT operating-loss history or a foreign tax credit needs coordinated carryover and allocation worksheets.",
    ),
    (
        "ISSUE_A_CURRENT_OR_CARRIED_FORM_3800_CREDIT_WITH_A_NEGATIVE_FORM_8978_LIN_7C3D87F0",
        "A current or carried Form 3800 credit with a negative Form 8978 line 14 needs a combined Form 3800 and Schedule 3 line 6l tax-capacity worksheet.",
    ),
    (
        "ISSUE_A_DIRECT_2026_PUBLIC_CHARITY_CASH_GIFT_ONE_ORDINARY_2026_PROPERTY_O_27AB2628",
        "A direct 2026 public-charity cash gift, one ordinary 2026 property or personal road-vehicle claim, and the full reconciled 2025 cash balance are estimated together only when their total fits within 50% of AGI. Above that boundary, amended section 170(b)(1)(G)(i) and retained section 170(b)(1)(G)(iii)(II) impose reciprocal A/G ceilings without settled 2026 allocation guidance; no deduction or carryover is reported as settled.",
    ),
    (
        "ISSUE_A_FARMING_NOL_REQUIRES_AN_EXPLICIT_VALID_CARRYBACK_WAIVER_OR_THE_SE_96CD586D",
        "A farming NOL requires an explicit valid carryback waiver or the separate 2024/2025 carryback computations. Missing, uncertain or ineligible elections never become an automatic waiver.",
    ),
    (
        "ISSUE_A_FILER_WHO_CAN_BE_CLAIMED_AS_A_DEPENDENT_BUT_IS_NOT_CLAIMED_RECONC_CEA36D49",
        "A filer who can be claimed as a dependent but is not claimed reconciles advance credit under Pub. 974's rules for an individual not included in a tax family (Form 8962 lines 11 to 23 column (f)), which is not calculated.",
    ),
    (
        "ISSUE_A_HOME_FIRST_USED_FOR_BUSINESS_BEFORE_MAY_13_1993_NEEDS_THE_PUB_946_C02327D8",
        "A home first used for business before May 13, 1993 needs the Pub 946 depreciation percentage (Form 8829 line 41) and is not calculated.",
    ),
    (
        "ISSUE_A_MONTH_COMBINING_A_POLICY_WITH_AN_ALLOCATED_BENCHMARK_AND_ANOTHER_0A7F77C8",
        "A month combining a policy with an allocated benchmark and another policy needs each coverage family's benchmark (Form 8962 Part IV column (f), lines 12 to 23 column (b)).",
    ),
    (
        "ISSUE_A_NEGATIVE_FORM_8978_ADJUSTMENT_BEYOND_FORM_1040_LINE_18_WITH_MSA_A_EBF0F0EA",
        "A negative Form 8978 adjustment beyond Form 1040 line 18 with MSA, ABLE, Trump-account earnings, mortgage-subsidy or education-credit additional tax needs a separate Chapter 1 Schedule 2 Part II classification. Only verified early-distribution and section 223 HSA additional taxes can currently absorb that excess; section 4973 HSA excess-contribution excise cannot.",
    ),
    (
        "ISSUE_A_PERSONALLY_USED_ROAD_VEHICLE_NEEDS_THE_REQUIRED_TIMELY_FORM_1098_B7819B5E",
        "A personally used road vehicle needs the required timely Form 1098-C or equivalent acknowledgment certifying its charity treatment. An unrelated sale uses the smaller of FMV and proceeds when proceeds exceed $500, or the smaller of FMV and $500 otherwise. Significant use, material improvement or needy-person transfer can use FMV; when similar road-vehicle claims exceed $5,000 together before income limits, each FMV-based exception vehicle needs a qualified appraisal and complete Form 8283 Section B evidence for its donee. Gross-proceeds-limited vehicles stay in Section A. Business use, appreciated basis or duplicated benefits need another worksheet.",
    ),
    (
        "ISSUE_A_POSITIVE_2025_50_CLASS_HOUSEHOLD_FURNISHING_CARRY_MEANS_THE_ORIGI_5D5542DE",
        "A positive 2025 50%-class household-furnishing carry means the original public-charity 50% room was exhausted before any 30%-class stock deduction. The paired single-stock 2025 carry must therefore equal its full original fair market value. Reconcile both amounts against the same filed 2025 class-limit worksheet.",
    ),
    (
        "ISSUE_A_PRIOR_30_CLASS_PUBLIC_STOCK_CARRYOVER_WITH_A_PRIOR_CASH_OR_OTHER_EF32A077",
        "A prior 30%-class public-stock carryover with a prior cash or other unsupported carryover class, current 30%-class student or whaling expense, or a 2026 capital-gain-property basis election needs a separate coordinated worksheet. One documented 2025 household-furnishing 50%-class carryover may be entered with this balance when the original-year combined worksheet is reconciled.",
    ),
    (
        "ISSUE_A_PRIOR_CASH_CARRYOVER_WITH_REGULAR_OR_AMT_OPERATING_LOSSES_OR_FORM_A1A1A4F4",
        "A prior cash carryover with regular or AMT operating losses or Form 1116 needs a coordinated carryover worksheet.",
    ),
    (
        "ISSUE_A_PRIOR_PUBLIC_STOCK_CARRYOVER_WITH_REGULAR_OR_AMT_OPERATING_LOSSES_669CC529",
        "A prior public-stock carryover with regular or AMT operating losses or Form 1116 requires a coordinated carryover worksheet.",
    ),
    (
        "ISSUE_A_PRIOR_SUSPENDED_PASSIVE_LOSS_FOR_A_PROPERTY_NOW_TREATED_AS_NONPAS_C9BD43A3",
        "A prior suspended passive loss for a property now treated as nonpassive under the real-estate-professional election needs its own release calculation before an estimate.",
    ),
    (
        "ISSUE_A_QUALIFIED_REAL_ESTATE_PROFESSIONAL_NEEDS_THE_CONFIRMED_PUB_925_MO_27DCF504",
        "A qualified real-estate professional needs the confirmed Pub 925 more-than-half and 750-hour tests before rental losses can be nonpassive.",
    ),
    (
        "ISSUE_A_REGULAR_AMT_NOL_HISTORY_OR_FORM_1116_CLAIM_WITH_NONCASH_GIFTS_NEE_E064CA92",
        "A regular/AMT NOL history or Form 1116 claim with noncash gifts needs separate loss-year or foreign-allocation worksheets.",
    ),
    (
        "ISSUE_A_SIMPLE_IRA_SOLO_401_K_DEFINED_BENEFIT_OR_OTHER_QUALIFIED_PLAN_CON_7F95EA79",
        "A SIMPLE IRA, solo 401(k), defined-benefit or other qualified-plan contribution for the owner (Schedule 1 line 16; Form 7206 line 9) is not calculated. Only a traditional owner-only SEP is.",
    ),
    (
        "ISSUE_A_SINGLE_RECONCILED_2025_60_CLASS_CASH_BALANCE_MAY_ACCOMPANY_ONE_DO_03A64A87",
        "A single reconciled 2025 60%-class cash balance may accompany one documented ordinary 2026 50%-class goods or personal road-vehicle gift, one documented long-term appreciated public-stock gift, or up to one documented 2026 hosted student and one whaling captain claim. A direct 2026 public-charity cash gift may also accompany any one of those supported sources. Ordinary property or a personal road vehicle with the older cash is estimated only when all current cash, the property claim, and the full older cash balance together fit within 50% of AGI; larger A/G mixes need a settled allocation rule. Current direct cash uses its 60% class before the older balance. In the stock pairing, appreciated stock then uses the residual 50% room and separate 30% limit; its excess keeps its 2026 gift year. The combined student and whaling 30% class instead uses its residual 50% limit and statutory 10%-of-AGI cash carve-out. Other prior vintages, multiple claims of either 30% kind, service expenses, QCD cash gifts, multiple property gifts and other imported classes need further coordination.",
    ),
    (
        "ISSUE_CAPITAL_CARRYOVERS_NOT_CONFIRMED_AS_INVESTMENT_LOSSES_NONBUSINESS_L_CC407A03",
        "Capital carryovers not confirmed as investment losses, nonbusiness loans with unresolved investment purpose need additional Form 4952 worksheets.",
    ),
    (
        "ISSUE_COMPLETE_ORDINARY_SCHEDULE_F_AVERAGING_RECORDS_ARE_REQUIRED_MIXED_Q_B04D2F54",
        "Farm income averaging with mixed QBI with an overall income cap or minimum-deduction allocation, itemized state/local income-tax allocation, prior NOLs, Archer MSA deductions, elected capital gains or young-filer tax needs further allocation worksheets.",
    ),
    (
        "ISSUE_CONFIRM_AN_OWNED_5_YEAR_VEHICLE_ON_SCHEDULE_C_WITH_ACTUAL_EXPENSES_93473DC7",
        "Confirm an owned 5-year vehicle on Schedule C or Schedule F with actual expenses and its actual vehicle design. New vehicles need more than 50% business use for section 179; a heavy passenger SUV without a design exception has a $32,000 election limit. Continuing passenger automobiles need a supported original schedule and annual cap history; unsupported changed-use, carryforward, lease and employer-provided cases need their own rules (Form 4562 Part V).",
    ),
    (
        "ISSUE_CONFIRM_EACH_DISTINCT_IRA_ACCOUNT_S_COMPLETE_2026_DESIGNATED_CONTRI_6FDFFFF9",
        "Confirm each distinct IRA account's complete 2026-designated contribution and all attributable net income or loss were corrected by the 2027 filing deadline, with an eligible extension if needed. A late return, another contribution year, or multiple corrections in one account needs separate treatment.",
    ),
    (
        "ISSUE_CONFIRM_ONE_EMPLOYEE_S_WRITTEN_2025_CALENDAR_YEAR_GRACE_BALANCE_AND_2A8798FD",
        "Confirm one employee's written 2025 calendar-year grace balance and current ordinary W-2 box 10 from the same plan, the service and reimbursement dates, and all distinct qualified care. Unmatched W-2, second plans, forfeiture, prior-year taxable wages or separate-return worksheets need review.",
    ),
    (
        "ISSUE_CONFIRM_PRIOR_YEAR_ELIGIBILITY_AND_RECONCILE_THE_PRIOR_RETURN_S_EXP_4999E5A2",
        "Reconcile the prior return's expense base and excluded/deducted benefits; unresolved history needs review.",
    ),
    (
        "ISSUE_CONFIRM_QUALIFYING_PURCHASED_EQUIPMENT_WITH_THE_ENTERED_SERVICE_DAT_EDE7B97E",
        "Confirm qualifying purchased equipment with the entered service date, first-year election or complete continuing history with identical regular/AMT basis or the separately recorded required AMT schedule. A first-use 2026 asset acquired before September 28, 2017 receives no bonus; ordinary 200% GDS for a 3-, 5-, 7-, or 10-year class uses the same basis, recovery period, and convention with 150% for AMT. Continuing earlier purchases require post-1998 service and 150%/straight-line GDS or a supported separate 200%/150% history. Unrecovered basis after the recovery period or an unsupported history needs its separate worksheet.",
    ),
    (
        "ISSUE_CONFIRM_THE_2025_EMPLOYER_PLAN_S_WRITTEN_CALENDAR_YEAR_GRACE_PERIOD_B4EA49C7",
        "Confirm the 2025 employer plan's written calendar-year grace period, its unused balance, the 2026 service and reimbursement dates, this employee's ownership, and all distinct qualified expenses. Current box 10 benefits, forfeiture, another plan and separate-return spouse worksheets need review.",
    ),
    (
        "ISSUE_CONFIRM_THE_2026_DEPOSITED_CONTRIBUTION_AND_DIRECT_TRUSTEE_TRANSFER_C8B72768",
        "Confirm the 2026-deposited contribution and direct trustee transfer by the ordinary 2026-return deadline, an actual filing extension, or the timely-filed-return corrective-action route. Confirm Form 1099-R code N for a 2026 transfer; reconcile a 2027 transfer with its information return when available. The final retained IRA total must include the original principal once in the destination account, not the source account.",
    ),
    (
        "ISSUE_CONFIRM_THE_BOX_14_CODE_I_STATEMENT_REPORTS_ONE_TRADE_OR_BUSINESS_O_DDB96AD3",
        "Confirm the box 14 code I statement reports one trade or business or aggregation for the entered boxes 6–8 activities, with no publicly traded partnership, cooperative or section 199A(g) items; other statements need their own Form 8995-A worksheet.",
    ),
    (
        "ISSUE_CONFIRM_THE_EMPLOYEE_PLAN_AND_OWNERSHIP_MULTI_FILER_SELF_EMPLOYED_B_DF3D2E4A",
        "A spouse's employer care benefits belong on this return only when filing jointly; multi-filer or self-employed benefit plans need additional worksheets.",
    ),
    (
        "ISSUE_CONFIRM_THE_K_1_BOXES_6_8_ACTIVITIES_BEFORE_THEY_ENTER_SCHEDULE_E_L_ABD611C3",
        "Confirm the K-1 boxes 6–8 activities before they enter Schedule E line 33: one fiduciary schedule per box with no loss, no passive-activity disposition attachment (Form 8582), no grouping with your own activities, box 9 deductions with identical regular, AMT and net-investment-income treatment (no box 12 code G, H or I), rentals without real-estate-professional treatment, and a box 6 business that does not trade financial instruments or commodities. Other patterns need their own Schedule E, Form 8582 and Form 8960 worksheets.",
    ),
    (
        "ISSUE_CONFIRM_THE_SAME_OWNER_HSA_DESTINATION_COMPLETE_ROLLOVER_HISTORY_60_8A899222",
        "Confirm the same-owner HSA destination, complete rollover history, 60-day deposit deadline and one-year limit; ineligible transfers need excess-contribution treatment.",
    ),
    (
        "ISSUE_CONFIRM_THE_SECTION_175_METHOD_QUALIFYING_PLAN_AND_COMPLETE_FARMING_85792609",
        "Confirm the section 175 method, qualifying plan and complete farming sources. Separate-return owners, nonproduction receipts, subsidies, other farming income, revised assessments and disposal/recapture need their source worksheets.",
    ),
    (
        "ISSUE_DEFERRAL_REQUIRES_CASH_ACCOUNTING_PROCEEDS_RECEIVED_FOR_2026_PHYSIC_34F9B266",
        "Deferral requires cash accounting and proceeds received for 2026 physical crop loss. Price-only or rainfall-index proceeds without physical damage cannot be deferred.",
    ),
    (
        "ISSUE_DISABILITY_ONSET_CANNOT_PRECEDE_THE_ACCOUNT_OWNER_S_BIRTH_93BC0663",
        "Disability onset cannot precede the account owner's birth.",
    ),
    (
        "ISSUE_EACH_MONTH_MARKED_AS_MARKETPLACE_COVERAGE_NEEDS_ITS_FORM_1095_A_MON_0495D93C",
        "Each month marked as Marketplace coverage needs its Form 1095-A month in Marketplace coverage (Pub. 974 Worksheet W line 1).",
    ),
    (
        "ISSUE_FORM_1040_LINE_26_THE_DATED_ESTIMATED_PAYMENTS_DO_NOT_ADD_UP_TO_THE_9C68C13C",
        "Form 1040 line 26: the dated estimated payments do not add up to the annual estimated payment amount.",
    ),
    (
        "ISSUE_FORM_1099_PATR_BOX_6_SECTION_199A_G_DEDUCTION_PASSED_THROUGH_FROM_A_2078FE70",
        "Form 1099-PATR box 6 section 199A(g) deduction passed through from a specified cooperative (Form 8995-A Part IV line 38) is not calculated.",
    ),
    (
        "ISSUE_FORM_1099_PATR_BOX_7_QUALIFIED_PAYMENTS_WITH_W_2_WAGES_ON_THE_FARM_0BB1F1A7",
        "Form 1099-PATR box 7 qualified payments with W-2 wages on the farm need the Form 8995-A Schedule D patron reduction (the lesser of 9% of the QBI allocable to the qualified payments or 50% of the allocable W-2 wages); that allocation is not calculated.",
    ),
    (
        "ISSUE_FORM_1099_PATR_BOX_9_SECTION_199A_A_SPECIFIED_SERVICE_TRADE_OR_BUSI_505016FB",
        "Form 1099-PATR box 9 section 199A(a) specified service trade or business items (Form 8995-A) are not calculated.",
    ),
    (
        "ISSUE_FORM_1099_PATR_DISTRIBUTIONS_NEED_AN_EXISTING_FARM_ACTIVITY_ON_THIS_90A69F86",
        "Form 1099-PATR distributions need an existing farm or farm rental on this return for the same owner before they can reach Schedule F line 3a/3b or Form 4835 line 2a/2b.",
    ),
    (
        "ISSUE_FORM_1099_PATR_DISTRIBUTIONS_NEED_A_CHOSEN_FARM",
        "Form 1099-PATR distributions need the farm or farm rental that receives them: choose one of this owner's farms on the form.",
    ),
    (
        "ISSUE_FORM_8582_CR_NEEDS_THE_FORM_8615_OR_SCHEDULE_J_TAX",
        "Passive credit limits use the return’s Form 8615 or Schedule J tax method, including the reduced-income comparison.",
    ),
    (
        "ISSUE_MARKETPLACE_COVERAGE_ESTABLISHED_UNDER_MORE_THAN_ONE_BUSINESS_NEEDS_A1BB156B",
        "Marketplace coverage established under more than one business needs the credit allocated among them (Pub. 974 Step 3, More than one trade or business).",
    ),
    (
        "ISSUE_MODIFIED_ADJUSTED_GROSS_INCOME_ABOVE_505_000_252_500_IF_MARRIED_FIL_95741BEC",
        "Modified adjusted gross income above $505,000 ($252,500 if married filing separately) with an actual-expense home office needs the Line 11 Worksheet iteration (Form 8829 line 11) and is not calculated.",
    ),
    (
        "ISSUE_ONLY_ONE_HOME_OFFICE_FIGURED_WITH_ACTUAL_EXPENSES_IS_CALCULATED_ON_FEEB163D",
        "Only one home office figured with actual expenses is calculated on a return, and not with a simplified-method office of the same owner (a second Form 8829 or the line 36 allocation between businesses).",
    ),
    (
        "ISSUE_POST_DEATH_DEEMED_DISTRIBUTIONS_BELONG_TO_THE_BENEFICIARY_S_ACCOUNT_64890807",
        "Post-death deemed distributions belong to the beneficiary's account treatment.",
    ),
    (
        "ISSUE_POST_DEATH_HSA_ROLLOVER_ACTIVITY_REQUIRES_BENEFICIARY_TREATMENT_93D3F469",
        "Post-death HSA rollover activity requires beneficiary treatment.",
    ),
    (
        "ISSUE_PROVIDE_ALL_TWELVE_MONTHS_FOR_EACH_FILER_WITH_A_STUDENT_OR_SELF_CAR_7C2A8D4D",
        "Provide all twelve months for each filer with a student or self-care exception.",
    ),
    (
        "ISSUE_RECONCILE_BOTH_SPOUSES_DISTINCT_ORDINARY_W_2_BOX_10_RECORDS_EMPLOYE_99A2C575",
        "Reconcile both spouses' distinct ordinary W-2 box 10 records, employer plan limits and qualified care costs. If a prior grace balance is present, confirm its employee, continuing plan, written grace period and care dates. Establish each employee's taxable line 26 share when the joint plan, expense or earned-income limit leaves taxable benefits; duplicate or incomplete sources need review.",
    ),
    (
        "ISSUE_RECONCILE_MONTHLY_W_2_ORDINARY_TAXABLE_FOREIGN_PAY_AND_ONE_DOCUMENT_AE79C11C",
        "Reconcile monthly W-2, ordinary taxable foreign pay and one documented cash-method U.S. Schedule C net amount separately with annual Income, and confirm student/self-care facts with People. Positive annual half-SE deduction, multiple or special businesses, partnership earnings, optional SE elections, disability-pension, Form 2555 exclusions/housing and unresolved source allocations need another worksheet.",
    ),
    (
        "ISSUE_SCHEDULE_F_LINE_12_THE_SECTION_175_GROSS_FARMING_INCOME_LIMIT_WITH_D6972ED9",
        "Schedule F line 12: the section 175 gross-farming-income limit with Form 1099-PATR cooperative distributions on the same farm is not calculated.",
    ),
    (
        "ISSUE_SCHEDULE_K_1_1065_BOX_13_CODE_A_OR_SCHEDULE_K_1_1120_S_BOX_12_CODE_C1503715",
        "Schedule K-1 (1065) box 13 code A or Schedule K-1 (1120-S) box 12 code A cash contributions together with a prior-year cash carryover need the combined carryover worksheet (Pub. 526 Worksheet 2), which this App does not calculate.",
    ),
    (
        "ISSUE_SELLING_YOUR_HOME_IN_CURRENT_YEAR_WITH_AN_ACTUAL_EXPENSE_HOME_OFFICE_NEEDS_C6369CE9",
        "Selling your home in 2026 with an actual-expense home office needs the depreciation up to the sale (Form 8829 line 41, Pub 946) and the business-use part of the sale (Pub 523), and is not calculated.",
    ),
    (
        "ISSUE_TAXABLE_ABLE_EARNINGS_COULD_CHANGE_THE_FORM_4952_DEDUCTION_OR_CARRY_1D0B701A",
        "Taxable ABLE, 529 or Coverdell earnings could change the Form 4952 deduction or carryforward; verify whether they count as investment income before completing this return.",
    ),
    (
        "ISSUE_TAXABLE_ADOPTION_BENEFITS_OR_NEGATIVE_WAGE_ADJUSTMENTS_FOR_A_FILER_F1EFDBA8",
        "Taxable adoption benefits or negative wage adjustments for a filer using monthly deemed earnings need monthly allocation.",
    ),
    (
        "ISSUE_THE_COMBINED_QUALIFIED_AMOUNT_FROM_DIFFERENT_PLANS_EXCEEDS_2_600_CR_15C8CEC6",
        "The combined qualified amount from different plans exceeds $2,600. Cross-plan treatment above that amount requires review before the early-distribution tax can be determined.",
    ),
    (
        "ISSUE_THE_FINAL_2026_FEDERAL_FUTA_CREDIT_REDUCTION_RATE_IS_REQUIRED_DO_NO_5042EA4F",
        "The final 2026 federal FUTA credit-reduction rate is required; do not substitute a potential or prior-year rate.",
    ),
    (
        "ISSUE_THE_PRIOR_RETURN_MINIMUM_TAX_CREDIT_NEEDS_COMPLETE_EXCLUSION_AND_RA_63A09D39",
        "The prior-return minimum-tax credit needs complete exclusion and rate records and unchanged credit ownership. Missing preferential worksheets, foreign treatment, loss histories and other special adjustments need additional work.",
    ),
    (
        "ISSUE_THE_SELECTED_STUDENT_MUST_BE_A_FILER_OR_AN_ELIGIBLE_DEPENDENT_ON_TH_879AF16D",
        "The selected student must be a filer or an eligible dependent on this return.",
    ),
    (
        "ISSUE_THIS_MONTHLY_WORKSHEET_REQUIRES_A_JOINT_RETURN_SEPARATE_RETURN_EXCE_C91E985C",
        "This monthly worksheet requires a joint return; separate-return exceptions need their additional worksheet.",
    ),
    (
        "ISSUE_THIS_NARROW_FORM_8978_ROUTE_NEEDS_A_SEPARATE_WORKSHEET_WHEN_THE_NEG_8ABBE1CD",
        "This narrow Form 8978 route needs a separate worksheet when the negative adjustment competes with child, other nonrefundable, or AMT foreign-tax credits.",
    ),
    (
        "ISSUE_THIS_ORDINARY_PERSONAL_PROPERTY_PATH_REQUIRES_ELIGIBLE_PUBLIC_CHARI_86353B77",
        "This ordinary personal-property path requires eligible public charities, good used condition for clothing and household items, reliable valuation and complete substantiation. If clothing, household goods, books or jewelry exceed $5,000 per similar-property group across all recipients before AGI limits, every affected gift needs a qualified appraisal covering it and complete Form 8283 Section B evidence for its donee. Other gifts use Section A when required. An inherited, appreciated, business, pass-through or special property gift needs another worksheet.",
    ),
    (
        "ISSUE_USE_ONE_CONSISTENT_BONUS_GDS_METHOD_ELECTION_FOR_EACH_RECOVERY_CLAS_5ABD1EF0",
        "Use one consistent bonus/GDS method election for each recovery class across all businesses and nonbusiness rentals on this return.",
    ),
    (
        "ISSUE_VACATION_OR_SHORT_TERM_RENTALS_LAND_ROYALTIES_SELF_RENTAL_AND_OTHER_CEF5E9C4",
        "Vacation or short-term rentals, land, royalties, self-rental and other Schedule E line 1b property types are not calculated by this rental entry; they are not calculated for 2026.",
    ),
    (
        "MESSAGE_ANSWER_NO_OR_YES_WHETHER_YOU_FILE_SEPARATELY_AS_A_VICTIM_OF_DOMESTIC_ABUSE_O",
        "Answer No or Yes: whether you file separately as a victim of domestic abuse or spousal abandonment (Form 8962 line A).",
    ),
    (
        "MESSAGE_A_DEPENDENT_S_AMOUNTS_ARE_OUTSIDE_THE_SUPPORTED_RANGE",
        "A dependent's amounts are outside the supported range.",
    ),
    (
        "MESSAGE_A_DEPENDENT_S_OWN_RETURN_APPLIES_ONLY_TO_SOMEONE_YOU_CLAIM_ON_THE_PEOPLE_PAG",
        "A dependent's own return applies only to someone you claim on the people page.",
    ),
    (
        "MESSAGE_A_FORMER_SPOUSE_S_SHARE_APPLIES_ONLY_WHEN_YOU_DO_NOT_FILE_JOINTLY_AND_A_SPOU",
        "A former spouse's share applies only when you do not file jointly, and a spouse's share only on a separate return (Form 8962 Part IV).",
    ),
    ("MESSAGE_A_SHARE_IS_A_PERCENTAGE_FROM_0_TO_100", "A share is a percentage from 0 to 100."),
    (
        "MESSAGE_A_STUDENT_IS_NOT_ON_THIS_RETURN_CHOOSE_THE_STUDENT_AGAIN",
        "A student is not on this return. Choose the student again.",
    ),
    (
        "MESSAGE_CHECK_THE_2025_CARRYOVER_THE_2025_GIFT_DATE_VALUE_ABOVE_BASIS_FOR_STOCK_OR_5",
        "Check the 2025 carryover: the 2025 gift date, value above basis for stock or $501–$5,000 for a household item, and a whole-dollar balance no more than the value.",
    ),
    (
        "MESSAGE_CHECK_THE_DATES_AND_AMOUNTS_OF_EACH_GIFT",
        "Check the dates and amounts of each gift.",
    ),
    ("DISPLAY_MARKETPLACE_STATEMENTS", "Forms 1095-A"),
    ("DISPLAY_MARKETPLACE_INSURER", "Insurance company"),
    ("DISPLAY_MARKETPLACE_COVERAGE", "Coverage and premiums"),
    ("DISPLAY_MARKETPLACE_SHARING", "Shared coverage"),
    (
        "EXPLAIN_MARKETPLACE_MARRIAGE",
        "Optional marriage-year worksheet: half joint household income, the actual permitted alternative family and its coverage months. This may reduce repayment but cannot create additional refundable credit.",
    ),
    ("LINE_MARKETPLACE_MARRIAGE_SPOUSE_CONTRIBUTION", "Marriage alternative: spouse contribution"),
    ("LINE_MARKETPLACE_MARRIAGE_SPOUSE_FAMILY", "Marriage alternative: spouse family"),
    ("LINE_MARKETPLACE_MARRIAGE_SPOUSE_START", "Marriage alternative: spouse start"),
    ("LINE_MARKETPLACE_MARRIAGE_SPOUSE_STOP", "Marriage alternative: spouse stop"),
    ("LINE_MARKETPLACE_MARRIAGE_YOU_CONTRIBUTION", "Marriage alternative: you contribution"),
    ("LINE_MARKETPLACE_MARRIAGE_YOU_FAMILY", "Marriage alternative: you family"),
    ("LINE_MARKETPLACE_MARRIAGE_YOU_START", "Marriage alternative: you start"),
    ("LINE_MARKETPLACE_MARRIAGE_YOU_STOP", "Marriage alternative: you stop"),
    ("MESSAGE_CHOOSE_EACH_SITUATION_ONLY_ONCE", "Choose each situation only once."),
    ("OFTEN_MISSED_ALL_TOPICS", "Not listed? See all topics or Search"),
    ("MESSAGE_SELECT_ITEMS_OR_NONE", "Select at least one item, or choose None of these."),
    ("MESSAGE_CHOOSE_ONLY_THE_LISTED_ITEMS", "Choose only the listed items."),
    ("MESSAGE_ENTER_AT_MOST_100_FORMS_3921", "Enter at most 100 Forms 3921."),
    (
        "MESSAGE_ENTER_AT_MOST_100_MISSED_REQUIRED_DISTRIBUTIONS",
        "Enter at most 100 missed required distributions.",
    ),
    ("MESSAGE_ENTER_AT_MOST_8_MOVES", "Enter at most 8 moves."),
    ("MESSAGE_ENTER_AT_MOST_20_CASUALTY_EVENTS", "Enter at most 20 casualty events."),
    ("MESSAGE_ENTER_AT_MOST_20_EARLIER_FORMS_8915_F", "Enter at most 20 earlier Forms 8915-F."),
    (
        "MESSAGE_ENTER_AT_MOST_20_EARLIER_YEAR_INSTALLMENT_SALES",
        "Enter at most 20 earlier-year installment sales.",
    ),
    ("MESSAGE_ENTER_AT_MOST_TEN_FORMS_1095_A", "Enter at most ten Forms 1095-A."),
    ("MESSAGE_ENTER_EACH_MONTH_FROM_1_TO_12", "Enter each month from 1 to 12."),
    (
        "MESSAGE_ENTER_EACH_MONTH_FROM_1_TO_12_WITH_AMOUNTS_IN_THE_SUPPORTED_RANGE",
        "Enter each month from 1 to 12 with amounts in the supported range.",
    ),
    (
        "MESSAGE_ENTER_EACH_MONTH_FROM_1_TO_12_WITH_A_BENCHMARK_IN_THE_SUPPORTED_RANGE",
        "Enter each month from 1 to 12 with a benchmark in the supported range.",
    ),
    (
        "MESSAGE_FORM_8912_YOUR_SPOUSE_S_TAX_CREDIT_BONDS_ARE_ENTERED_ONLY_ON_A_JOINT_RETURN",
        "Form 8912: your spouse's tax credit bonds are entered only on a joint return.",
    ),
    (
        "MESSAGE_FORM_2555_ENTER_VALID_HOUSING_AMOUNTS_AND_2026_STAY_DATES",
        "Form 2555: enter valid housing expenses, allowances, household amounts and 2026 stay dates.",
    ),
    ("MESSAGE_SHARED_MONTHS_ARE_MONTHS_1_TO_12", "Shared months are months 1 to 12."),
    (
        "MESSAGE_THE_PEOPLE_ON_THIS_RETURN_CHANGED_REOPEN_THIS_PAGE_BEFORE_SAVING",
        "The people on this return changed. Reopen this page before saving.",
    ),
    (
        "MESSAGE_THIS_SITUATION_BELONGS_TO_ANOTHER_SECTION",
        "This situation belongs to another section.",
    ),
    (
        "PAYMENT_ESTIMATED_SOURCE_UNAVAILABLE",
        "The annual estimated payment amount is unavailable. Retain the actual dated payment sources.",
    ),
    (
        "MESSAGE_YOUR_SPOUSE_S_EXPENSES_BELONG_ON_THIS_RETURN_ONLY_WHEN_YOU_FILE_JOINTLY",
        "Your spouse's expenses belong on this return only when you file jointly.",
    ),
    ("PROBLEM_SECTION_UNAVAILABLE_YET", "This section is not available yet."),
    (
        "PROBLEM_UNDISPLAYABLE_CHARITY_GIFT_OR_CARRYOVER_RECORD",
        "This return holds a charity gift or carryover record this page cannot show yet. Use a fresh development task.",
    ),
    (
        "PROBLEM_UNDISPLAYABLE_EDUCATION_CREDIT_RECORD",
        "This return holds an education credit record this page cannot show yet. Use a fresh development task.",
    ),
    (
        "PROBLEM_UNDISPLAYABLE_HSA_FAMILY_ALLOCATION",
        "This return holds an HSA family allocation this page cannot show yet. Use a fresh development task.",
    ),
    (
        "PROBLEM_UNDISPLAYABLE_HSA_RECORD",
        "This return holds an HSA record this page cannot show yet. Use a fresh development task.",
    ),
    (
        "PROBLEM_UNDISPLAYABLE_INVESTMENT_INTEREST_RECORD",
        "This return holds an investment interest record this page cannot show yet. Use a fresh development task.",
    ),
    (
        "PROBLEM_UNDISPLAYABLE_MARKETPLACE_RECORD",
        "This return holds a Marketplace record this page cannot show yet. Use a fresh development task.",
    ),
    (
        "PROBLEM_UNDISPLAYABLE_MORTGAGE_RECORD",
        "This return holds a mortgage record this page cannot show yet. Use a fresh development task.",
    ),
    (
        "PROBLEM_UNDISPLAYABLE_OPTIONAL_SELF_EMPLOYMENT_ELECTION",
        "This return holds an optional self-employment election this page cannot show yet. Use a fresh development task.",
    ),
    ("SAVING_OWNER_YOURS", "yours"),
    ("SAVING_OWNER_SPOUSE", "your spouse's"),
    (
        "SAVING_REMOVES_YOUR_CHILD_AND_DEPENDENT_CARE_CLAIM",
        "Saving removes your child and dependent care claim.",
    ),
    ("SAVING_REMOVES_YOUR_SPOUSE_S_CARE_EXPENSES", "Saving removes your spouse's care expenses."),
    ("SAVING_KIND_EDUCATION_CREDIT_ONE", "education credit"),
    ("SAVING_KIND_EDUCATION_CREDIT_MANY", "education credits"),
    ("SAVING_KIND_FORM_3921_MANY", "Forms 3921"),
    ("SAVING_KIND_OTHER_DOCUMENT_ONE", "other document"),
    ("SAVING_KIND_OTHER_DOCUMENT_MANY", "other documents"),
    ("SAVING_KIND_SITUATION_ONE", "situation"),
    ("SAVING_KIND_SITUATION_MANY", "situations"),
    ("SAVING_KIND_TAX_CREDIT_BOND_ONE", "tax credit bond"),
    ("SAVING_KIND_TAX_CREDIT_BOND_MANY", "tax credit bonds"),
    (
        "K1_1041_BOX_9_ABOVE_BOX_6",
        "Box 9 deductions for the business activity cannot exceed box 6 income.",
    ),
    (
        "K1_1041_BOX_9_ABOVE_BOX_7",
        "Box 9 deductions for the rental real estate activity cannot exceed box 7 income.",
    ),
    (
        "K1_1041_BOX_9_ABOVE_BOX_8",
        "Box 9 deductions for the other rental activity cannot exceed box 8 income.",
    ),
    (
        "K1_1041_CODE_F_ABOVE_BOX_6",
        "Box 14 code F gross farming and fishing income cannot exceed box 6 income.",
    ),
    (
        "K1_1041_QBI_ABOVE_ACTIVITY_INCOME",
        "Qualified business income cannot exceed boxes 6–8 after their box 9 deductions.",
    ),
    (
        "EXPLAIN_ESTATE_ACTIVITY_COLUMN",
        "Estate or trust document {0}, activity {1}: Schedule E column {2}.",
    ),
    (
        "EXPLAIN_ESTATE_ACTIVITY_CARRY",
        "Estate or trust document {0}, activity {1}: passive loss still suspended.",
    ),
    ("LABEL_K1_ACTIVITY_SOURCE", "{0}"),
    (
        "K1_ACTIVITY_SOURCE_AMOUNT_INVALID",
        "Check the amounts, source codes and retained records for this K-1.",
    ),
    (
        "K1_QBI_CURRENT_ABOVE_COMPONENT_SOURCE",
        "Reconcile the current qualified amounts with their actual component source.",
    ),
    (
        "K1_ACTIVITY_CURRENT_SOURCE_RECONCILE",
        "Reconcile the actual activity and entity-only current amounts with the original K-1 controls.",
    ),
    (
        "ISSUE_K1_ACTIVITY_OWNER_OUTPUT_UNAVAILABLE",
        "This estimate does not yet calculate the combined basis, at-risk and passive limits for these activity records.",
    ),
    (
        "ESTATE_CREDIT_LINE_INVALID",
        "Choose a Form 3800 line this credit code can report: line 1c or 4i for code I, line 1f or 4e for code J.",
    ),
    ("ESTATE_FOREIGN_AMOUNT_INVALID", "Review the fiduciary's foreign-tax statement amount."),
    ("DISPLAY_ESTATE_GAINS", "Gains and royalties"),
    ("DISPLAY_ESTATE_STATEMENTS", "Additional statements"),
    ("DISPLAY_ESTATE_FINAL", "Final year and other boxes"),
    ("DISPLAY_ESTATE_ACTIVITY_RECORDS", "Participation and prior losses"),
    (
        "WARN_ESTATE_NOL_PERIOD_LEFT_OUT",
        "Original fiduciary loss year {0}: the remaining beneficiary carry period or actual fiduciary-period history is unknown, so this year's inherited NOL benefit is left out. Other income and supported loss years remain.",
    ),
    (
        "WARN_ESTATE_NOL_POOL_LEFT_OUT",
        "Original fiduciary loss year {0}: competing sources have different finite periods or AMT exception character. Their inherited loss portion is left out pending an actual source allocation; ordinary income and independently determined loss portions remain.",
    ),
    (
        "ESTATE_AMT_ACTIVITY_RECONCILE",
        "Reconcile each activity's AMT difference with the allocated box 12 adjustments.",
    ),
    (
        "EXPLAIN_ESTATE_AMT_INDEPENDENT",
        "Fiduciary source adjustment on its applicable Form 6251 line, included once.",
    ),
    (
        "ISSUE_ESTATE_AMT_SOURCE_UNAVAILABLE",
        "The fiduciary's box 12 adjustment needs its actual regular/AMT source and applicable Form 6251 owner under sections 56 and 57; its added tax cannot be omitted.",
    ),
    ("LINE_ESTATE_TRUST_N_AMT_INDEPENDENT_N", "Estate/trust {0}: independent AMT source {1}"),
    (
        "WARN_ESTATE_ACTIVITY_LOSS_LEFT_OUT",
        "Activity {0}: unresolved participation or beneficiary loss classification; its deductions and carried losses are left out while known positive income remains.",
    ),
    (
        "WARN_ESTATE_UNENTERED_CREDIT_LEFT_OUT",
        "The unidentified Form 1041 credit benefit is left out. Enter any credit recapture separately as an added tax.",
    ),
    (
        "WARN_ESTATE_CREDIT_LEFT_OUT",
        "Form 1041 box 13 code {0}: the credit to holders of tax credit bonds (Form 8912) is figured only for bonds you hold directly, so this credit is left out; ordinary income and tax remain, and your tax may be lower.",
    ),
    (
        "WARN_ESTATE_CREDIT_ACTIVITY_LEFT_OUT",
        "Form 1041 box 13 code {0}: its actual activity allocation is needed before applying passive-credit limits; only this credit is left out.",
    ),
    (
        "WARN_ESTATE_FOREIGN_ALLOCATION_LEFT_OUT",
        "The fiduciary foreign-tax statement needs full country, category and regular/AMT allocations; the foreign-credit benefit is left out.",
    ),
    (
        "WARN_ESTATE_FINAL_ENTITLEMENT_LEFT_OUT",
        "Final-year deductions and carryovers require estate/trust termination and beneficiary succession; these benefits are left out.",
    ),
    (
        "WARN_ESTATE_FINAL_OTHER_LEFT_OUT",
        "Final-year deduction {0} needs its owning deduction worksheet and is left out.",
    ),
    ("INT_PRIVATE_ACTIVITY_ABOVE_EXEMPT", "Box 9 cannot exceed box 8 tax-exempt interest."),
    (
        "OID_PRIVATE_ACTIVITY_ABOVE_EXEMPT",
        "The private-activity part cannot exceed box 11 tax-exempt original issue discount.",
    ),
    ("FOREIGN_QUALIFIED_ABOVE_INCOME", "Qualified dividends cannot exceed this source's income."),
    ("NOL_NOTHING_ENTERED", "Enter at least one loss year, regular or alternative minimum tax."),
    (
        "RETAINED_RESEARCH_PARTS_ABOVE_BALANCE",
        "The identified business portions must be positive and cannot exceed this retained balance.",
    ),
    (
        "RETAINED_RESEARCH_SOURCE_TWICE",
        "Enter each originating business only once for this retained balance.",
    ),
    (
        "RETAINED_RESEARCH_SOURCE_UNAVAILABLE",
        "Select an actual eligible business on this return for each retained source portion.",
    ),
    ("LINE_NOL_N_PTP_QUALIFIED_DEDUCTED", "NOL {0}: qualified PTP loss actually deducted"),
    (
        "LINE_NOL_N_PTP_QUALIFIED_PRE_DEBT_CLOSING",
        "NOL {0}: qualified PTP loss still suspended before debt reductions",
    ),
    (
        "LINE_NOL_N_PTP_QUALIFIED_LOST",
        "NOL {0}: qualified PTP character no longer carried without a deduction",
    ),
    (
        "EXPLAIN_NOL_PTP_EBL_PRE_DEBT_CLOSING",
        "Qualified PTP character remaining in the financial NOL before any section 108 debt-attribute reduction. This balance is outside the Form 8995 loss carry.",
    ),
    (
        "EXPLAIN_NOL_PTP_EBL_LOST",
        "Original PTP qualification no longer backed by the financial NOL after absorption, expiration or its whole-dollar boundary, without an actual NOL deduction. It enters neither this year's Form 8995 loss nor a later qualified carry.",
    ),
    (
        "EXPLAIN_NOL_PTP_EBL_QUALIFIED",
        "The actual NOL deduction includes this original-year qualified PTP loss once, separately from ordinary QBI and from the prior Form 8995 loss carry.",
    ),
    ("NOL_YEAR_TWICE", "Enter each loss year only once in each list."),
    (
        "EXPLAIN_INJURED_SPOUSE_LIMITS",
        "This is a prospective allocation before later IRS offsets, refunds or debt balances. The IRS makes the final computation. Your ordinary federal estimate remains available when a Form 8379 allocation cannot be calculated.",
    ),
    ("INJURED_COMPONENT_OTHER_ADJUSTMENTS", "Other adjustments attributable to both spouses"),
    (
        "EXPLAIN_FORM8379_QBI_ALLOCATION",
        "Supplemental QBI deduction allocated from the accepted joint deduction and actual source ownership; this amount is separate from printed Form 8379 line 15.",
    ),
    (
        "INJURED_COMPONENT_OTHER_TAXES",
        "Other additional taxes and credit repayments attributable to the injured spouse",
    ),
    (
        "INJURED_SOURCE_FACTS_NOT_REQUESTED",
        "These ownership answers apply only within a requested injured-spouse claim.",
    ),
    (
        "INJURED_SPOUSE_WITHHOLDING_MANAGEMENT_NEEDED",
        "Identify the Texas management classification for the community funds that supplied this withholding.",
    ),
    (
        "WARN_8379_ADOPTION_OWNER_UNAVAILABLE",
        "The Form 8379 refund estimate needs the final adoption credit by adopting filer. Your ordinary federal estimate is retained.",
    ),
    (
        "INJURED_SPOUSE_COMMUNITY_ABOVE_PAYMENT",
        "The community part must be within this payment, with an applicable source-management classification.",
    ),
    (
        "MARKETPLACE_ADVANCE_ON_UNSUBSIDIZED_SHARE",
        "A policy shared without advance credit cannot report an advance credit.",
    ),
    (
        "MARKETPLACE_SHARE_MONTHS_REVERSED",
        "The first month shared cannot be after the last month shared.",
    ),
    (
        "MARKETPLACE_ISSUER_TWICE",
        "Enter each Form 1095-A once; add the line 2 policy number to tell two policies from the same issuer apart.",
    ),
    ("MARKETPLACE_MONTH_TWICE", "Enter each month only once."),
    ("MARKETPLACE_PERSON_TWICE", "Enter each person only once."),
    (
        "MARKETPLACE_SHARE_OUTSIDE_POLICY",
        "The months shared must be months this Form 1095-A covers.",
    ),
    (
        "MARKETPLACE_GAP_MONTH_NOT_COVERED",
        "A month someone could not take the credit must be a month a Form 1095-A covers.",
    ),
    (
        "MARKETPLACE_FAMILY_BENCHMARK_NEEDED",
        "Two of your forms show different column B amounts for the same month. Enter your family's benchmark for that month.",
    ),
    (
        "MARKETPLACE_FAMILY_BENCHMARK_NOT_NEEDED",
        "Enter a family benchmark only for a month in which two forms show different column B amounts.",
    ),
    (
        "EXEMPT_REDUCTION_ABOVE_AMOUNT",
        "Exempt premium must belong to and not exceed its matching exempt OID.",
    ),
    ("DIV_897_ORDINARY_ABOVE_BOX", "Box 2e section 897 ordinary dividends cannot exceed box 1a."),
    ("DIV_897_CAPITAL_ABOVE_BOX", "Box 2f section 897 capital gain cannot exceed box 2a."),
    ("SALE_WASH_SALE_ABOVE_LOSS", "A disallowed wash-sale loss cannot exceed this sale's loss."),
    (
        "PATR_PERSONAL_ABOVE_DISTRIBUTIONS",
        "Personal-purchase dividends cannot exceed boxes 1, 2 and 3.",
    ),
    ("REFUND_DEDUCTION_NEEDED", "Choose how the refunded tax was deducted."),
    (
        "RENTAL_HOME_CARRIES_NEEDED",
        "Enter the carries from last year's home-use worksheet for this rental.",
    ),
    (
        "RENTAL_SECTION179_BUSINESS_NEEDED",
        "Answer whether you actively conduct this rental as a trade or business.",
    ),
    (
        "EXPLAIN_EMPLOYEE_COVERAGE_PAYROLL",
        "Employee payroll coverage worksheet: known covered pay and its employee Social Security and Medicare liability, using separate coverage and the remaining Social Security wage base. This is an estimate, not a numbered Form 8919.",
    ),
    (
        "WARN_EMPLOYEE_RAILROAD_WITHHOLDING_UNAVAILABLE",
        "{0}: actual railroad withholding is unavailable. Its unverified credit is left out; known compensation and tax calculations remain included.",
    ),
    (
        "EMPLOYEE_PAYROLL_RANGE",
        "The combined employee payroll amounts exceed the supported monetary range. Correct the amounts on Wages.",
    ),
    (
        "LINE_RETIREMENT_N_LTC_INVESTMENT_REDUCTION",
        "Retirement source {0}: actual care-charge investment reduction",
    ),
    (
        "EXPLAIN_RETIREMENT_N_LTC_INVESTMENT_REDUCTION",
        "Qualified Form 1099-R code W cash-value charges are excluded under section 72(e)(11); a permitted code 6 exchange is also excluded. Gross stays in this original source and is not reported as pension income or derived as a deductible premium. Box 8a is the actual investment reduction, including zero, not the full charge/exchange value. Another withdrawal or discharge uses its documented already-adjusted investment; the App never subtracts this reduction twice. A post-annuitization exclusion-ratio adjustment needs actual insurer tax results or a separately verified worksheet; this charge alone does not compute it.",
    ),
    (
        "ISSUE_BOX_10_BASIS_UNKNOWN",
        "The part of box 10 that was not taxable when rolled in decides the recapture amount for the 10% additional tax; without it that tax is not calculated.",
    ),
    (
        "NOTE_1099R_EARLIER_YEAR",
        "{0}: this amount is taxable for 2025 or earlier, not 2026; report it on that year's return, amending it if it is already filed. Its withholding counts for 2026.",
    ),
    (
        "NOTE_1099R_RECHARACTERIZED",
        "{0}: attach a statement explaining the recharacterization; enter the contribution as the kind of IRA it became on the IRA contributions page.",
    ),
    (
        "WARN_DIRECT_PLAN_BASIS_NOT_TRACKED",
        "{0}: the receiving plan does not keep your after-tax amount separately, so it may be taxed again when paid out; ask the plan to account for it.",
    ),
    (
        "DISASTER_ABOVE_LIMIT",
        "Qualified disaster distributions for one disaster, with earlier years, cannot exceed $22,000 (Form 8915-F line 1e).",
    ),
    (
        "ISO_EXERCISED_BEFORE_GRANTED",
        "Form 3921: the date exercised (box 2) cannot be before the date granted (box 1).",
    ),
    (
        "RENTAL_PROFESSIONAL_NEEDS_STATUS",
        "Record your real estate professional status on About you (your spouse's on About your spouse) before choosing it for a rental.",
    ),
    (
        "K3_WITHHOLDING_NEEDS_EACH_PASSIVE_COUNTRY",
        "Give the withholding treatment of every country that has a passive row.",
    ),
    ("K1_CREDIT_CODE_TWICE", "Enter each K-1 credit code once."),
    (
        "EXPLAIN_EMPLOYEE_EQUIPMENT_SECTION179",
        "Current section 179 deduction after the return-wide dollar and business-income limits. Current elections precede prior carryforwards, oldest year first. Included once in this job's Form 2106 line 4 expenses.",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_SECTION179",
        "Form 2106 job {0} equipment {1} section 179 deduction",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_SECTION179_CARRYFORWARD",
        "Form 2106 job {0} equipment {1} section 179 carryforward",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_DISPOSAL_N_RECAPTURE",
        "Employee expenses {0} equipment {1} disposal {2} recapture",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_DISPOSAL_ADDITIONAL_DEPRECIATION",
        "Employee expenses {0} equipment {1} disposal additional depreciation",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_DISPOSAL_ADJUSTED_BASIS",
        "Employee expenses {0} equipment {1} disposal adjusted basis",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_DISPOSAL_COST",
        "Employee expenses {0} equipment {1} disposal cost",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_DISPOSAL_DEPRECIATION",
        "Employee expenses {0} equipment {1} disposal depreciation",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_DISPOSAL_FORM6252_ADJUSTED_BASIS",
        "Employee expenses {0} equipment {1} disposal Form 6252 adjusted basis",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_DISPOSAL_FORM6252_ASSUMED_DEBT",
        "Employee expenses {0} equipment {1} disposal Form 6252 assumed debt",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_DISPOSAL_FORM6252_CONTRACT_PRICE",
        "Employee expenses {0} equipment {1} disposal Form 6252 contract price",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_DISPOSAL_FORM6252_COST",
        "Employee expenses {0} equipment {1} disposal Form 6252 cost",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_DISPOSAL_FORM6252_DEEMED_PAYMENT",
        "Employee expenses {0} equipment {1} disposal Form 6252 deemed payment",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_DISPOSAL_FORM6252_DEPRECIATION",
        "Employee expenses {0} equipment {1} disposal Form 6252 depreciation",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_DISPOSAL_FORM6252_EXCESS_DEBT",
        "Employee expenses {0} equipment {1} disposal Form 6252 excess debt",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_DISPOSAL_FORM6252_EXPENSES",
        "Employee expenses {0} equipment {1} disposal Form 6252 expenses",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_DISPOSAL_FORM6252_GAIN",
        "Employee expenses {0} equipment {1} disposal Form 6252 gain",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_DISPOSAL_FORM6252_GROSS_PROFIT",
        "Employee expenses {0} equipment {1} disposal Form 6252 gross profit",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_DISPOSAL_FORM6252_PROFIT",
        "Employee expenses {0} equipment {1} disposal Form 6252 profit before exclusion",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_DISPOSAL_FORM6252_PERCENTAGE",
        "Employee expenses {0} equipment {1} disposal Form 6252 gross profit percentage",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_DISPOSAL_FORM6252_INCOME",
        "Employee expenses {0} equipment {1} disposal Form 6252 income",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_DISPOSAL_FORM6252_NET_PRICE",
        "Employee expenses {0} equipment {1} disposal Form 6252 net price",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_DISPOSAL_FORM6252_ORDINARY",
        "Employee expenses {0} equipment {1} disposal Form 6252 ordinary",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_DISPOSAL_FORM6252_PAYMENTS",
        "Employee expenses {0} equipment {1} disposal Form 6252 payments",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_DISPOSAL_FORM6252_PRICE",
        "Employee expenses {0} equipment {1} disposal Form 6252 price",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_DISPOSAL_FORM6252_RECAPTURE",
        "Employee expenses {0} equipment {1} disposal Form 6252 recapture",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_DISPOSAL_FORM6252_TOTAL",
        "Employee expenses {0} equipment {1} disposal Form 6252 total",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_DISPOSAL_FORM6252_TOTAL_PAYMENTS",
        "Employee expenses {0} equipment {1} disposal Form 6252 total payments",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_DISPOSAL_FORM6252_UNRECAPTURED_N",
        "Employee expenses {0} equipment {1} disposal Form 6252 unrecaptured {2}",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_DISPOSAL_GAIN",
        "Employee expenses {0} equipment {1} disposal gain",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_DISPOSAL_INSTALLMENT_GAIN",
        "Employee expenses {0} equipment {1} disposal installment gain",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_DISPOSAL_INSTALLMENT_ORDINARY",
        "Employee expenses {0} equipment {1} disposal Form 6252 line 26 to Form 4797 line 11",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_DISPOSAL_ORDINARY",
        "Employee expenses {0} equipment {1} disposal ordinary",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_DISPOSAL_POSTPONED",
        "Employee expenses {0} equipment {1} disposal gain postponed under section 1033",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_DISPOSAL_SALE_PRICE",
        "Employee expenses {0} equipment {1} disposal sale price",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_DISPOSAL_SECTION1231",
        "Employee expenses {0} equipment {1} disposal section 1231",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_PARTIAL_DEPRECIATION",
        "Form 2106 job {0} equipment {1} partial disposition depreciation allowed on the part",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_N_PARTIAL_GAIN",
        "Form 2106 job {0} equipment {1} partial disposition gain or loss",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EQUIPMENT_DEPRECIATION_CLASS_N",
        "Form 2106 job {0} equipment depreciation, Form 4562 line {1}",
    ),
    ("WARN_ASSET_MACRS_RECOVERY_LEFT_OUT", "{0}: {1}"),
    (
        "INTEREST_LIMIT_ELECTION_SOURCE",
        "Correct this activity’s actual election kind and effective tax year.",
    ),
    (
        "ASSET_ADS_ORIGINAL_HISTORY_UNAVAILABLE",
        "The original recovery system or actual prior depreciation records are unavailable. No depreciation benefit is included for the unresolved source.",
    ),
    (
        "ASSET_NONMACRS_FINANCIAL_UNAVAILABLE",
        "The actual property sources exceed the signed-cent financial carrier. Retain the original amounts; this estimate is unavailable.",
    ),
    (
        "ASSET_NONMACRS_SOURCE_FACTS",
        "The original non-MACRS source facts do not reconcile with this property's dates, basis, use and deductions. Correct the actual schedule or output records.",
    ),
    (
        "ASSET_NONMACRS_ORIGINAL_HISTORY",
        "The original non-MACRS schedule or basis history is needed. An unexplained balance does not restart its original recovery period.",
    ),
    ("LINE_BUSINESS_N_DEPRECIATION_CLASS_N", "Business {0}: depreciation, Form 4562 line {1}"),
    ("LINE_RENTAL_N_DEPRECIATION_CLASS_N", "Rental {0}: depreciation, Form 4562 line {1}"),
    (
        "LINE_PERSONAL_RENTAL_N_DEPRECIATION_CLASS_N",
        "Equipment rental {0}: depreciation, Form 4562 line {1}",
    ),
    (
        "EXPLAIN_BUSINESS_N_ASSET_N_N_NONMACRS_DETAILS",
        "Original method: {0}. Applicable recovery period: {1} months, using {2}. Current recovery retains the original schedule, basis and actual prior deductions.",
    ),
    (
        "EXPLAIN_BUSINESS_N_ASSET_N_N_NONMACRS",
        "Recovery from the actual original non-MACRS schedule and prior basis history; statutory elections and published annual rounding are applied once.",
    ),
    (
        "HOME_OFFICE_AREA_ABOVE_HOME",
        "The office's square feet cannot exceed the whole home's square feet.",
    ),
    (
        "HOME_OFFICE_LAND_ABOVE_BASIS",
        "The part for the land cannot exceed what the home cost or its value.",
    ),
    (
        "ASSET_179_ABOVE_SUV_LIMIT",
        "This heavy passenger SUV has no listed design exception. Its section 179 election cannot exceed $32,000 for 2026; eligible bonus and regular depreciation may still apply to the remaining cost.",
    ),
    (
        "ASSET_BONUS_ELECTION_BY_CLASS",
        "Electing out of bonus depreciation applies to all 2026 property of the same kind; choose regular depreciation only for every such asset on this return, or for none.",
    ),
    (
        "VEHICLE_PERCENT_FROM_MILES",
        "For a vehicle, the business use percentage must equal business miles divided by all miles, rounded down to a whole percent.",
    ),
    ("VEHICLE_BUSINESS_MILES_ABOVE_TOTAL", "Business miles cannot exceed all miles driven."),
    (
        "WARN_METHOD_CHANGE_LEFT_OUT",
        "{0}: this section 481(a) adjustment (a different AMT amount, other consent terms, or a rental you also used personally) isn't supported, so it is left out of this estimate; your tax may be higher or lower.",
    ),
    (
        "EXPLAIN_METHOD_CHANGE_INCREASE",
        "The 2026 part of the section 481(a) increase from the {0,number,0} accounting-method change ({1}); other income on this activity's schedule.",
    ),
    (
        "EXPLAIN_METHOD_CHANGE_DECREASE",
        "The section 481(a) decrease from the {0,number,0} accounting-method change ({1}), deducted in full in its year of change; other expenses on this activity's schedule.",
    ),
    (
        "EXPLAIN_METHOD_CHANGE_REMAINING",
        "The rest of this section 481(a) increase, taken in 2027 and later years unless the activity stops sooner.",
    ),
    ("LINE_BUSINESS_N_METHOD_CHANGE_N", "Business {0} accounting-method change {1}"),
    (
        "LINE_BUSINESS_N_METHOD_CHANGE_N_REMAINING",
        "Business {0} accounting-method change {1} remaining",
    ),
    ("LINE_FARM_N_METHOD_CHANGE_N", "Farm {0} accounting-method change {1}"),
    ("LINE_FARM_N_METHOD_CHANGE_N_REMAINING", "Farm {0} accounting-method change {1} remaining"),
    ("LINE_RENTAL_N_METHOD_CHANGE_N", "Rental {0} accounting-method change {1}"),
    (
        "LINE_RENTAL_N_METHOD_CHANGE_N_REMAINING",
        "Rental {0} accounting-method change {1} remaining",
    ),
    ("MESSAGE_ENTER_AT_MOST_10_FORMS_2106", "Enter at most 10 Forms 2106."),
    ("EXPLAIN_EMPLOYEE_EXPENSES_N_LINE_N", "A line of Form 2106 for one job, in whole dollars."),
    (
        "EXPLAIN_EMPLOYEE_EXPENSES_N_VEHICLE_N",
        "A vehicle's operating costs (line 27) or depreciation (line 28) on Form 2106 Part II.",
    ),
    (
        "EXPLAIN_EMPLOYEE_EXPENSES_N_EXCESS",
        "Reimbursements above the expenses (Form 2106 line 8), included in Form 1040 line 1a.",
    ),
    (
        "EXPLAIN_SCHEDULE_1_12",
        "Reservist, performing-artist and fee-basis official business expenses (Form 2106).",
    ),
    ("LINE_EMPLOYEE_EXPENSES_N_LINE_N", "Form 2106 job {0} line {1}"),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_OPERATING",
        "Form 2106 job {0} vehicle {1} operating costs",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DEPRECIATION",
        "Form 2106 job {0} vehicle {1} depreciation",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_EXCESS_REIMBURSEMENT",
        "Form 2106 job {0} reimbursements above expenses",
    ),
    ("LINE_SCHEDULE_N_EMPLOYEE_BUSINESS_EXPENSES", "Schedule {0} employee business expenses"),
    (
        "MORTGAGE_AVERAGE_NEEDED",
        "Your mortgages' box 2 balances together are above the Pub 936 limit; choose an average-balance method for each loan instead of the default.",
    ),
    ("VEHICLE_GIFT_FACTS_NEEDED", "Complete the personal gift facts linked to this Form 1098-C."),
    (
        "STUDENT_LOAN_EMPLOYER_ABOVE_INTEREST",
        "The interest your employer paid cannot be more than the student loan interest you entered.",
    ),
    (
        "GIFT_GOODS_SECTION_B",
        "One kind of clothing, household items, books or jewelry totals more than $5,000 for 2026. That needs a qualified appraisal and Form 8283 Section B, which this App does not calculate yet.",
    ),
    (
        "VEHICLE_SECTION_B",
        "Your vehicle claims total more than $5,000 and a vehicle is claimed at fair market value (box 5a or 5b). That needs a qualified appraisal and Form 8283 Section B, which this App does not calculate yet.",
    ),
    (
        "MORTGAGE_GIFT_OWNER_REQUIRED",
        "Form 8828: select whose interest in this home was given away. A joint return with recapture tax needs the taxpayer, spouse or both owners.",
    ),
    (
        "VEHICLE_BUSINESS_ABOVE_INTEREST",
        "The part deducted on Schedule C, E or F cannot be more than the interest you paid on this loan.",
    ),
    ("CASUALTY_EVENT_TWICE", "Give each casualty event a different name."),
    (
        "CASUALTY_AFTER_ABOVE_BEFORE",
        "The value after the casualty cannot be more than the value before it.",
    ),
    (
        "CASUALTY_RECEIVED_ABOVE_REIMBURSEMENT",
        "The part you received in 2026 cannot be more than the reimbursement you received or expect.",
    ),
    ("GIFT_PRIOR_STOCK_TWICE", "Enter one 2025 stock carryover; this App calculates only one."),
    (
        "GIFT_PRIOR_HOUSEHOLD_TWICE",
        "Enter one 2025 household item carryover; this App calculates only one.",
    ),
    // Review and hub row statuses (ReviewItem.status). The argument-free keys
    // are the rows' fixed states, which display.json compares by key.
    // A hub's topic list title: its section's name.
    ("SECTION_INCOME", "Income"),
    ("SECTION_ADJUSTMENTS", "Adjustments to income"),
    ("SECTION_ITEMIZED", "Itemized deductions"),
    ("SECTION_CREDITS", "Credits"),
    ("REVIEW_NOT_STARTED", "Not started"),
    ("REVIEW_NONE", "None"),
    ("REVIEW_NONE_OF_THESE", "None of these"),
    ("REVIEW_RECORDED", "Recorded"),
    ("REVIEW_NEEDS_ATTENTION", "Needs attention"),
    ("REVIEW_NO_CARE_COSTS", "No care costs"),
    ("REVIEW_PEOPLE", "{0,plural,one {# person} other {# people}}"),
    ("REVIEW_CARE_COSTS", "Care costs for {0,plural,one {# person} other {# people}}"),
    (
        "LINE_FOREIGN_AMT_BRANCH_N_APPORTIONED_DEDUCTIONS",
        "AMT Form 1116 foreign branch category, {0}: line 3g, apportioned deductions",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_N_DEDUCTIONS_SUBTOTAL",
        "AMT Form 1116 foreign branch category, {0}: line 3c, deductions subtotal",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_N_FOREIGN_LOSSES",
        "AMT Form 1116 foreign branch category, {0}: line 5, foreign losses",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_N_GROSS_FOREIGN_INCOME",
        "AMT Form 1116 foreign branch category, {0}: line 3d, gross foreign income",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_N_GROSS_INCOME",
        "AMT Form 1116 foreign branch category, {0}: line 1a, gross income",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_N_GROSS_INCOME_ALL_SOURCES",
        "AMT Form 1116 foreign branch category, {0}: line 3e, gross income from all sources",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_N_GROSS_INCOME_RATIO",
        "AMT Form 1116 foreign branch category, {0}: line 3f, ratio of foreign to all income",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_N_ITEMIZED_OR_STANDARD",
        "AMT Form 1116 foreign branch category, {0}: line 3a, itemized or standard deduction",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_N_MORTGAGE_INTEREST",
        "AMT Form 1116 foreign branch category, {0}: line 4a, home mortgage interest",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_N_OTHER_DEDUCTIONS",
        "AMT Form 1116 foreign branch category, {0}: line 3b, other deductions",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_N_OTHER_INTEREST",
        "AMT Form 1116 foreign branch category, {0}: line 4b, other interest",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_N_RELATED_EXPENSES",
        "AMT Form 1116 foreign branch category, {0}: line 2, definitely related expenses",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_N_TAXABLE_INCOME",
        "AMT Form 1116 foreign branch category, {0}: line 7, taxable income",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_N_TAXES",
        "AMT Form 1116 foreign branch category, {0}: Part II foreign taxes",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_N_TOTAL_DEDUCTIONS",
        "AMT Form 1116 foreign branch category, {0}: line 6, total deductions",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_CARRYOVER",
        "AMT Form 1116 foreign branch category: line 10, carryover",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_CATEGORY_CREDIT",
        "AMT Form 1116 foreign branch category: line 24, category credit",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_CREDIT_LIMITATION",
        "AMT Form 1116 foreign branch category: line 23, credit limitation",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_FOREIGN_INCOME",
        "AMT Form 1116 foreign branch category: line 15, foreign income",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_HIGH_TAX_KICKOUT",
        "AMT Form 1116 foreign branch category: line 13, high-tax kickout",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_HTKO_GROSS_INCOME",
        "AMT Form 1116 foreign branch category, high-taxed income column: line 1a, gross income",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_HTKO_TAXABLE_INCOME",
        "AMT Form 1116 foreign branch category, high-taxed income column: line 7, taxable income",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_HTKO_TOTAL_DEDUCTIONS",
        "AMT Form 1116 foreign branch category, high-taxed income column: line 6, total deductions",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_INCOME_ADJUSTMENTS",
        "AMT Form 1116 foreign branch category: line 16, income adjustments",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_LIMITATION_INCREASE",
        "AMT Form 1116 foreign branch category: line 22, limitation increase",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_LIMITATION_RATIO",
        "AMT Form 1116 foreign branch category: line 19, limitation ratio",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_MAXIMUM_CREDIT",
        "AMT Form 1116 foreign branch category: line 21, maximum credit",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_NET_FOREIGN_INCOME",
        "AMT Form 1116 foreign branch category: line 17, net foreign income",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_OVERALL_DOMESTIC_LOSS_NEW",
        "Foreign amt branch overall domestic loss new",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_OVERALL_DOMESTIC_LOSS_RECAPTURED",
        "Foreign amt branch overall domestic loss recaptured",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_OVERALL_DOMESTIC_LOSS_REMAINING",
        "Foreign amt branch overall domestic loss remaining",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_OVERALL_FOREIGN_LOSS_NEW",
        "Foreign amt branch overall foreign loss new",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_OVERALL_FOREIGN_LOSS_RECAPTURED",
        "Foreign amt branch overall foreign loss recaptured",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_OVERALL_FOREIGN_LOSS_REMAINING",
        "Foreign amt branch overall foreign loss remaining",
    ),
    ("LINE_FOREIGN_AMT_BRANCH_SEPARATE_LOSS_NETTED", "Foreign amt branch separate loss netted"),
    ("LINE_FOREIGN_AMT_BRANCH_SEPARATE_LOSS_NEW", "Foreign amt branch separate loss new"),
    (
        "LINE_FOREIGN_AMT_BRANCH_SEPARATE_LOSS_RECAPTURED",
        "Foreign amt branch separate loss recaptured",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_SEPARATE_LOSS_REMAINING",
        "Foreign amt branch separate loss remaining",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_TAX_REDUCTION",
        "AMT Form 1116 foreign branch category: line 12, tax reduction",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_TAXES_AVAILABLE",
        "AMT Form 1116 foreign branch category: line 14, taxes available",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_TAXES_FOR_CATEGORY",
        "AMT Form 1116 foreign branch category: line 9, taxes for the category",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_TAXES_TOTAL",
        "AMT Form 1116 foreign branch category: line 8, foreign taxes",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_TAXES_WITH_CARRYOVER",
        "AMT Form 1116 foreign branch category: line 11, taxes with carryover",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_TOTAL_GROSS_INCOME",
        "AMT Form 1116 foreign branch category, total column: line 1a, gross income",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_TOTAL_TAXABLE_INCOME",
        "AMT Form 1116 foreign branch category, total column: line 7, taxable income",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_TOTAL_TOTAL_DEDUCTIONS",
        "AMT Form 1116 foreign branch category, total column: line 6, total deductions",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_US_INCOME_TAX",
        "AMT Form 1116 foreign branch category: line 20, U.S. income tax",
    ),
    (
        "LINE_FOREIGN_AMT_BRANCH_WORLDWIDE_INCOME",
        "AMT Form 1116 foreign branch category: line 18, worldwide income",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_N_APPORTIONED_DEDUCTIONS",
        "AMT Form 1116 general category, {0}: line 3g, apportioned deductions",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_N_DEDUCTIONS_SUBTOTAL",
        "AMT Form 1116 general category, {0}: line 3c, deductions subtotal",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_N_FOREIGN_LOSSES",
        "AMT Form 1116 general category, {0}: line 5, foreign losses",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_N_GROSS_FOREIGN_INCOME",
        "AMT Form 1116 general category, {0}: line 3d, gross foreign income",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_N_GROSS_INCOME",
        "AMT Form 1116 general category, {0}: line 1a, gross income",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_N_GROSS_INCOME_ALL_SOURCES",
        "AMT Form 1116 general category, {0}: line 3e, gross income from all sources",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_N_GROSS_INCOME_RATIO",
        "AMT Form 1116 general category, {0}: line 3f, ratio of foreign to all income",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_N_ITEMIZED_OR_STANDARD",
        "AMT Form 1116 general category, {0}: line 3a, itemized or standard deduction",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_N_MORTGAGE_INTEREST",
        "AMT Form 1116 general category, {0}: line 4a, home mortgage interest",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_N_OTHER_DEDUCTIONS",
        "AMT Form 1116 general category, {0}: line 3b, other deductions",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_N_OTHER_INTEREST",
        "AMT Form 1116 general category, {0}: line 4b, other interest",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_N_RELATED_EXPENSES",
        "AMT Form 1116 general category, {0}: line 2, definitely related expenses",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_N_TAXABLE_INCOME",
        "AMT Form 1116 general category, {0}: line 7, taxable income",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_N_TAXES",
        "AMT Form 1116 general category, {0}: Part II foreign taxes",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_N_TOTAL_DEDUCTIONS",
        "AMT Form 1116 general category, {0}: line 6, total deductions",
    ),
    ("LINE_FOREIGN_AMT_GENERAL_CARRYOVER", "AMT Form 1116 general category: line 10, carryover"),
    (
        "LINE_FOREIGN_AMT_GENERAL_CATEGORY_CREDIT",
        "AMT Form 1116 general category: line 24, category credit",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_CREDIT_LIMITATION",
        "AMT Form 1116 general category: line 23, credit limitation",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_FOREIGN_INCOME",
        "AMT Form 1116 general category: line 15, foreign income",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_HIGH_TAX_KICKOUT",
        "AMT Form 1116 general category: line 13, high-tax kickout",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_HTKO_GROSS_INCOME",
        "AMT Form 1116 general category, high-taxed income column: line 1a, gross income",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_HTKO_TAXABLE_INCOME",
        "AMT Form 1116 general category, high-taxed income column: line 7, taxable income",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_HTKO_TOTAL_DEDUCTIONS",
        "AMT Form 1116 general category, high-taxed income column: line 6, total deductions",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_INCOME_ADJUSTMENTS",
        "AMT Form 1116 general category: line 16, income adjustments",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_LIMITATION_INCREASE",
        "AMT Form 1116 general category: line 22, limitation increase",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_LIMITATION_RATIO",
        "AMT Form 1116 general category: line 19, limitation ratio",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_MAXIMUM_CREDIT",
        "AMT Form 1116 general category: line 21, maximum credit",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_NET_FOREIGN_INCOME",
        "AMT Form 1116 general category: line 17, net foreign income",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_TAX_REDUCTION",
        "AMT Form 1116 general category: line 12, tax reduction",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_TAXES_AVAILABLE",
        "AMT Form 1116 general category: line 14, taxes available",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_TAXES_FOR_CATEGORY",
        "AMT Form 1116 general category: line 9, taxes for the category",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_TAXES_TOTAL",
        "AMT Form 1116 general category: line 8, foreign taxes",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_TAXES_WITH_CARRYOVER",
        "AMT Form 1116 general category: line 11, taxes with carryover",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_TOTAL_GROSS_INCOME",
        "AMT Form 1116 general category, total column: line 1a, gross income",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_TOTAL_TAXABLE_INCOME",
        "AMT Form 1116 general category, total column: line 7, taxable income",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_TOTAL_TOTAL_DEDUCTIONS",
        "AMT Form 1116 general category, total column: line 6, total deductions",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_US_INCOME_TAX",
        "AMT Form 1116 general category: line 20, U.S. income tax",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_WORLDWIDE_INCOME",
        "AMT Form 1116 general category: line 18, worldwide income",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_N_APPORTIONED_DEDUCTIONS",
        "AMT Form 1116 passive category, {0}: line 3g, apportioned deductions",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_N_DEDUCTIONS_SUBTOTAL",
        "AMT Form 1116 passive category, {0}: line 3c, deductions subtotal",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_N_FOREIGN_LOSSES",
        "AMT Form 1116 passive category, {0}: line 5, foreign losses",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_N_GROSS_FOREIGN_INCOME",
        "AMT Form 1116 passive category, {0}: line 3d, gross foreign income",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_N_GROSS_INCOME",
        "AMT Form 1116 passive category, {0}: line 1a, gross income",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_N_GROSS_INCOME_ALL_SOURCES",
        "AMT Form 1116 passive category, {0}: line 3e, gross income from all sources",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_N_GROSS_INCOME_RATIO",
        "AMT Form 1116 passive category, {0}: line 3f, ratio of foreign to all income",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_N_ITEMIZED_OR_STANDARD",
        "AMT Form 1116 passive category, {0}: line 3a, itemized or standard deduction",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_N_MORTGAGE_INTEREST",
        "AMT Form 1116 passive category, {0}: line 4a, home mortgage interest",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_N_OTHER_DEDUCTIONS",
        "AMT Form 1116 passive category, {0}: line 3b, other deductions",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_N_OTHER_INTEREST",
        "AMT Form 1116 passive category, {0}: line 4b, other interest",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_N_RELATED_EXPENSES",
        "AMT Form 1116 passive category, {0}: line 2, definitely related expenses",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_N_TAXABLE_INCOME",
        "AMT Form 1116 passive category, {0}: line 7, taxable income",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_N_TAXES",
        "AMT Form 1116 passive category, {0}: Part II foreign taxes",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_N_TOTAL_DEDUCTIONS",
        "AMT Form 1116 passive category, {0}: line 6, total deductions",
    ),
    ("LINE_FOREIGN_AMT_PASSIVE_CARRYOVER", "AMT Form 1116 passive category: line 10, carryover"),
    (
        "LINE_FOREIGN_AMT_PASSIVE_CATEGORY_CREDIT",
        "AMT Form 1116 passive category: line 24, category credit",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_CREDIT_LIMITATION",
        "AMT Form 1116 passive category: line 23, credit limitation",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_FOREIGN_INCOME",
        "AMT Form 1116 passive category: line 15, foreign income",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_HIGH_TAX_KICKOUT",
        "AMT Form 1116 passive category: line 13, high-tax kickout",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_HTKO_GROSS_INCOME",
        "AMT Form 1116 passive category, high-taxed income column: line 1a, gross income",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_HTKO_TAXABLE_INCOME",
        "AMT Form 1116 passive category, high-taxed income column: line 7, taxable income",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_HTKO_TOTAL_DEDUCTIONS",
        "AMT Form 1116 passive category, high-taxed income column: line 6, total deductions",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_INCOME_ADJUSTMENTS",
        "AMT Form 1116 passive category: line 16, income adjustments",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_LIMITATION_INCREASE",
        "AMT Form 1116 passive category: line 22, limitation increase",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_LIMITATION_RATIO",
        "AMT Form 1116 passive category: line 19, limitation ratio",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_MAXIMUM_CREDIT",
        "AMT Form 1116 passive category: line 21, maximum credit",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_NET_FOREIGN_INCOME",
        "AMT Form 1116 passive category: line 17, net foreign income",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_TAX_REDUCTION",
        "AMT Form 1116 passive category: line 12, tax reduction",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_TAXES_AVAILABLE",
        "AMT Form 1116 passive category: line 14, taxes available",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_TAXES_FOR_CATEGORY",
        "AMT Form 1116 passive category: line 9, taxes for the category",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_TAXES_TOTAL",
        "AMT Form 1116 passive category: line 8, foreign taxes",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_TAXES_WITH_CARRYOVER",
        "AMT Form 1116 passive category: line 11, taxes with carryover",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_TOTAL_GROSS_INCOME",
        "AMT Form 1116 passive category, total column: line 1a, gross income",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_TOTAL_TAXABLE_INCOME",
        "AMT Form 1116 passive category, total column: line 7, taxable income",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_TOTAL_TOTAL_DEDUCTIONS",
        "AMT Form 1116 passive category, total column: line 6, total deductions",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_US_INCOME_TAX",
        "AMT Form 1116 passive category: line 20, U.S. income tax",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_WORLDWIDE_INCOME",
        "AMT Form 1116 passive category: line 18, worldwide income",
    ),
    (
        "LINE_FOREIGN_AMT_SANCTIONED_N_APPORTIONED_DEDUCTIONS",
        "AMT Form 1116 section 901(j) category, {0}: line 3g, apportioned deductions",
    ),
    (
        "LINE_FOREIGN_AMT_SANCTIONED_N_DEDUCTIONS_SUBTOTAL",
        "AMT Form 1116 section 901(j) category, {0}: line 3c, deductions subtotal",
    ),
    (
        "LINE_FOREIGN_AMT_SANCTIONED_N_FOREIGN_LOSSES",
        "AMT Form 1116 section 901(j) category, {0}: line 5, foreign losses",
    ),
    (
        "LINE_FOREIGN_AMT_SANCTIONED_N_GROSS_FOREIGN_INCOME",
        "AMT Form 1116 section 901(j) category, {0}: line 3d, gross foreign income",
    ),
    (
        "LINE_FOREIGN_AMT_SANCTIONED_N_GROSS_INCOME",
        "AMT Form 1116 section 901(j) category, {0}: line 1a, gross income",
    ),
    (
        "LINE_FOREIGN_AMT_SANCTIONED_N_GROSS_INCOME_ALL_SOURCES",
        "AMT Form 1116 section 901(j) category, {0}: line 3e, gross income from all sources",
    ),
    (
        "LINE_FOREIGN_AMT_SANCTIONED_N_GROSS_INCOME_RATIO",
        "AMT Form 1116 section 901(j) category, {0}: line 3f, ratio of foreign to all income",
    ),
    (
        "LINE_FOREIGN_AMT_SANCTIONED_N_ITEMIZED_OR_STANDARD",
        "AMT Form 1116 section 901(j) category, {0}: line 3a, itemized or standard deduction",
    ),
    (
        "LINE_FOREIGN_AMT_SANCTIONED_N_MORTGAGE_INTEREST",
        "AMT Form 1116 section 901(j) category, {0}: line 4a, home mortgage interest",
    ),
    (
        "LINE_FOREIGN_AMT_SANCTIONED_N_OTHER_DEDUCTIONS",
        "AMT Form 1116 section 901(j) category, {0}: line 3b, other deductions",
    ),
    (
        "LINE_FOREIGN_AMT_SANCTIONED_N_OTHER_INTEREST",
        "AMT Form 1116 section 901(j) category, {0}: line 4b, other interest",
    ),
    (
        "LINE_FOREIGN_AMT_SANCTIONED_N_RELATED_EXPENSES",
        "AMT Form 1116 section 901(j) category, {0}: line 2, definitely related expenses",
    ),
    (
        "LINE_FOREIGN_AMT_SANCTIONED_N_TAXABLE_INCOME",
        "AMT Form 1116 section 901(j) category, {0}: line 7, taxable income",
    ),
    (
        "LINE_FOREIGN_AMT_SANCTIONED_N_TAXES",
        "AMT Form 1116 section 901(j) category, {0}: Part II foreign taxes",
    ),
    (
        "LINE_FOREIGN_AMT_SANCTIONED_N_TOTAL_DEDUCTIONS",
        "AMT Form 1116 section 901(j) category, {0}: line 6, total deductions",
    ),
    (
        "LINE_FOREIGN_AMT_SANCTIONED_FOREIGN_INCOME",
        "AMT Form 1116 section 901(j) category: line 15, foreign income",
    ),
    (
        "LINE_FOREIGN_AMT_SANCTIONED_HTKO_GROSS_INCOME",
        "AMT Form 1116 section 901(j) category, high-taxed income column: line 1a, gross income",
    ),
    (
        "LINE_FOREIGN_AMT_SANCTIONED_HTKO_TAXABLE_INCOME",
        "AMT Form 1116 section 901(j) category, high-taxed income column: line 7, taxable income",
    ),
    (
        "LINE_FOREIGN_AMT_SANCTIONED_HTKO_TOTAL_DEDUCTIONS",
        "AMT Form 1116 section 901(j) category, high-taxed income column: line 6, total deductions",
    ),
    (
        "LINE_FOREIGN_AMT_SANCTIONED_INCOME_ADJUSTMENTS",
        "AMT Form 1116 section 901(j) category: line 16, income adjustments",
    ),
    (
        "LINE_FOREIGN_AMT_SANCTIONED_NET_FOREIGN_INCOME",
        "AMT Form 1116 section 901(j) category: line 17, net foreign income",
    ),
    (
        "LINE_FOREIGN_AMT_SANCTIONED_OVERALL_DOMESTIC_LOSS_NEW",
        "Foreign amt sanctioned overall domestic loss new",
    ),
    (
        "LINE_FOREIGN_AMT_SANCTIONED_OVERALL_FOREIGN_LOSS_NEW",
        "Foreign amt sanctioned overall foreign loss new",
    ),
    ("LINE_FOREIGN_AMT_SANCTIONED_SEPARATE_LOSS_NEW", "Foreign amt sanctioned separate loss new"),
    (
        "LINE_FOREIGN_AMT_SANCTIONED_TAXES_TOTAL",
        "AMT Form 1116 section 901(j) category: line 8, foreign taxes",
    ),
    (
        "LINE_FOREIGN_AMT_SANCTIONED_TOTAL_GROSS_INCOME",
        "AMT Form 1116 section 901(j) category, total column: line 1a, gross income",
    ),
    (
        "LINE_FOREIGN_AMT_SANCTIONED_TOTAL_TAXABLE_INCOME",
        "AMT Form 1116 section 901(j) category, total column: line 7, taxable income",
    ),
    (
        "LINE_FOREIGN_AMT_SANCTIONED_TOTAL_TOTAL_DEDUCTIONS",
        "AMT Form 1116 section 901(j) category, total column: line 6, total deductions",
    ),
    ("LINE_FOREIGN_AMT_WORKSHEET_A_LINE_N", "Foreign AMT capital Worksheet A line {0}"),
    ("LINE_FOREIGN_AMT_WORKSHEET_B_LINE_N", "Foreign AMT capital Worksheet B line {0}"),
    (
        "LINE_FOREIGN_BRANCH_N_APPORTIONED_DEDUCTIONS",
        "Form 1116 foreign branch category, {0}: line 3g, apportioned deductions",
    ),
    (
        "LINE_FOREIGN_BRANCH_N_DEDUCTIONS_SUBTOTAL",
        "Form 1116 foreign branch category, {0}: line 3c, deductions subtotal",
    ),
    (
        "LINE_FOREIGN_BRANCH_N_FOREIGN_LOSSES",
        "Form 1116 foreign branch category, {0}: line 5, foreign losses",
    ),
    (
        "LINE_FOREIGN_BRANCH_N_GROSS_FOREIGN_INCOME",
        "Form 1116 foreign branch category, {0}: line 3d, gross foreign income",
    ),
    (
        "LINE_FOREIGN_BRANCH_N_GROSS_INCOME",
        "Form 1116 foreign branch category, {0}: line 1a, gross income",
    ),
    (
        "LINE_FOREIGN_BRANCH_N_GROSS_INCOME_ALL_SOURCES",
        "Form 1116 foreign branch category, {0}: line 3e, gross income from all sources",
    ),
    (
        "LINE_FOREIGN_BRANCH_N_GROSS_INCOME_RATIO",
        "Form 1116 foreign branch category, {0}: line 3f, ratio of foreign to all income",
    ),
    (
        "LINE_FOREIGN_BRANCH_N_ITEMIZED_OR_STANDARD",
        "Form 1116 foreign branch category, {0}: line 3a, itemized or standard deduction",
    ),
    (
        "LINE_FOREIGN_BRANCH_N_MORTGAGE_INTEREST",
        "Form 1116 foreign branch category, {0}: line 4a, home mortgage interest",
    ),
    (
        "LINE_FOREIGN_BRANCH_N_OTHER_DEDUCTIONS",
        "Form 1116 foreign branch category, {0}: line 3b, other deductions",
    ),
    (
        "LINE_FOREIGN_BRANCH_N_OTHER_INTEREST",
        "Form 1116 foreign branch category, {0}: line 4b, other interest",
    ),
    (
        "LINE_FOREIGN_BRANCH_N_RELATED_EXPENSES",
        "Form 1116 foreign branch category, {0}: line 2, definitely related expenses",
    ),
    (
        "LINE_FOREIGN_BRANCH_N_TAXABLE_INCOME",
        "Form 1116 foreign branch category, {0}: line 7, taxable income",
    ),
    (
        "LINE_FOREIGN_BRANCH_N_TAXES",
        "Form 1116 foreign branch category, {0}: Part II foreign taxes",
    ),
    (
        "LINE_FOREIGN_BRANCH_N_TOTAL_DEDUCTIONS",
        "Form 1116 foreign branch category, {0}: line 6, total deductions",
    ),
    (
        "LINE_FOREIGN_BRANCH_AMT_N_CARRY_ADJUSTED",
        "Schedule B (Form 1116) AMT foreign branch category, {0} column: line 3, adjusted carryover",
    ),
    (
        "LINE_FOREIGN_BRANCH_AMT_N_CARRY_EXPIRED",
        "Schedule B (Form 1116) AMT foreign branch category, {0} column: line 5, carryover expired",
    ),
    (
        "LINE_FOREIGN_BRANCH_AMT_N_CARRY_OPENING",
        "Schedule B (Form 1116) AMT foreign branch category, {0} column: line 1, prior carryover",
    ),
    (
        "LINE_FOREIGN_BRANCH_AMT_N_CARRY_REMAINING",
        "Schedule B (Form 1116) AMT foreign branch category, {0} column: line 8, carryover remaining",
    ),
    (
        "LINE_FOREIGN_BRANCH_AMT_N_CARRY_USED",
        "Schedule B (Form 1116) AMT foreign branch category, {0} column: line 4, carryover used",
    ),
    (
        "LINE_FOREIGN_BRANCH_AMT_N_CARRYBACK_ADJUSTMENT",
        "Schedule B (Form 1116) AMT foreign branch category, {0} column: line 2a, carryback adjustment",
    ),
    (
        "LINE_FOREIGN_BRANCH_AMT_N_OTHER_ADJUSTMENTS",
        "Schedule B (Form 1116) AMT foreign branch category, {0} column: line 2c to 2g, other adjustments",
    ),
    (
        "LINE_FOREIGN_BRANCH_AMT_N_REDETERMINATION",
        "Schedule B (Form 1116) AMT foreign branch category, {0} column: line 2b, section 905(c) redeterminations",
    ),
    (
        "LINE_FOREIGN_BRANCH_AMT_CARRY_ADJUSTED_TOTAL",
        "Schedule B (Form 1116) AMT foreign branch category: line 3, total adjusted carryover",
    ),
    (
        "LINE_FOREIGN_BRANCH_AMT_CARRY_EXPIRED",
        "Schedule B (Form 1116) AMT foreign branch category: line 5, total carryover expired",
    ),
    (
        "LINE_FOREIGN_BRANCH_AMT_CARRY_OPENING_TOTAL",
        "Schedule B (Form 1116) AMT foreign branch category: line 1, total prior carryover",
    ),
    (
        "LINE_FOREIGN_BRANCH_AMT_CARRY_USED",
        "Schedule B (Form 1116) AMT foreign branch category: line 4, total carryover used",
    ),
    (
        "LINE_FOREIGN_BRANCH_AMT_CARRYBACK_N",
        "Schedule B (Form 1116) AMT foreign branch category: line 7, carryback to {0}",
    ),
    (
        "LINE_FOREIGN_BRANCH_AMT_DEDUCTION_YEAR_ABSORBED_N",
        "Schedule B (Form 1116) AMT foreign branch category: line 7, absorbed in deduction year {0}",
    ),
    (
        "LINE_FOREIGN_BRANCH_AMT_NEW_CARRYFORWARD",
        "Schedule B (Form 1116) AMT foreign branch category: line 8, new carryforward",
    ),
    (
        "LINE_FOREIGN_BRANCH_AMT_RETAINED_CARRYFORWARD",
        "Schedule B (Form 1116) AMT foreign branch category: line 8, total carryforward retained",
    ),
    (
        "LINE_FOREIGN_BRANCH_AMT_UNUSED_TAX",
        "Schedule B (Form 1116) AMT foreign branch category: line 6, excess foreign taxes",
    ),
    ("LINE_FOREIGN_BRANCH_CARRYOVER", "Form 1116 foreign branch category: line 10, carryover"),
    (
        "LINE_FOREIGN_BRANCH_CATEGORY_CREDIT",
        "Form 1116 foreign branch category: line 24, category credit",
    ),
    (
        "LINE_FOREIGN_BRANCH_CREDIT_LIMITATION",
        "Form 1116 foreign branch category: line 23, credit limitation",
    ),
    (
        "LINE_FOREIGN_BRANCH_FOREIGN_INCOME",
        "Form 1116 foreign branch category: line 15, foreign income",
    ),
    (
        "LINE_FOREIGN_BRANCH_HIGH_TAX_KICKOUT",
        "Form 1116 foreign branch category: line 13, high-tax kickout",
    ),
    (
        "LINE_FOREIGN_BRANCH_HTKO_GROSS_INCOME",
        "Form 1116 foreign branch category, high-taxed income column: line 1a, gross income",
    ),
    (
        "LINE_FOREIGN_BRANCH_HTKO_TAXABLE_INCOME",
        "Form 1116 foreign branch category, high-taxed income column: line 7, taxable income",
    ),
    (
        "LINE_FOREIGN_BRANCH_HTKO_TOTAL_DEDUCTIONS",
        "Form 1116 foreign branch category, high-taxed income column: line 6, total deductions",
    ),
    (
        "LINE_FOREIGN_BRANCH_INCOME_ADJUSTMENTS",
        "Form 1116 foreign branch category: line 16, income adjustments",
    ),
    (
        "LINE_FOREIGN_BRANCH_LIMITATION_INCREASE",
        "Form 1116 foreign branch category: line 22, limitation increase",
    ),
    (
        "LINE_FOREIGN_BRANCH_LIMITATION_RATIO",
        "Form 1116 foreign branch category: line 19, limitation ratio",
    ),
    (
        "LINE_FOREIGN_BRANCH_MAXIMUM_CREDIT",
        "Form 1116 foreign branch category: line 21, maximum credit",
    ),
    (
        "LINE_FOREIGN_BRANCH_NET_FOREIGN_INCOME",
        "Form 1116 foreign branch category: line 17, net foreign income",
    ),
    (
        "LINE_FOREIGN_BRANCH_REGULAR_N_CARRY_ADJUSTED",
        "Schedule B (Form 1116) foreign branch category, {0} column: line 3, adjusted carryover",
    ),
    (
        "LINE_FOREIGN_BRANCH_REGULAR_N_CARRY_EXPIRED",
        "Schedule B (Form 1116) foreign branch category, {0} column: line 5, carryover expired",
    ),
    (
        "LINE_FOREIGN_BRANCH_REGULAR_N_CARRY_OPENING",
        "Schedule B (Form 1116) foreign branch category, {0} column: line 1, prior carryover",
    ),
    (
        "LINE_FOREIGN_BRANCH_REGULAR_N_CARRY_REMAINING",
        "Schedule B (Form 1116) foreign branch category, {0} column: line 8, carryover remaining",
    ),
    (
        "LINE_FOREIGN_BRANCH_REGULAR_N_CARRY_USED",
        "Schedule B (Form 1116) foreign branch category, {0} column: line 4, carryover used",
    ),
    (
        "LINE_FOREIGN_BRANCH_REGULAR_N_CARRYBACK_ADJUSTMENT",
        "Schedule B (Form 1116) foreign branch category, {0} column: line 2a, carryback adjustment",
    ),
    (
        "LINE_FOREIGN_BRANCH_REGULAR_N_OTHER_ADJUSTMENTS",
        "Schedule B (Form 1116) foreign branch category, {0} column: line 2c to 2g, other adjustments",
    ),
    (
        "LINE_FOREIGN_BRANCH_REGULAR_N_REDETERMINATION",
        "Schedule B (Form 1116) foreign branch category, {0} column: line 2b, section 905(c) redeterminations",
    ),
    (
        "LINE_FOREIGN_BRANCH_REGULAR_CARRY_ADJUSTED_TOTAL",
        "Schedule B (Form 1116) foreign branch category: line 3, total adjusted carryover",
    ),
    (
        "LINE_FOREIGN_BRANCH_REGULAR_CARRY_EXPIRED",
        "Schedule B (Form 1116) foreign branch category: line 5, total carryover expired",
    ),
    (
        "LINE_FOREIGN_BRANCH_REGULAR_CARRY_OPENING_TOTAL",
        "Schedule B (Form 1116) foreign branch category: line 1, total prior carryover",
    ),
    (
        "LINE_FOREIGN_BRANCH_REGULAR_CARRY_USED",
        "Schedule B (Form 1116) foreign branch category: line 4, total carryover used",
    ),
    (
        "LINE_FOREIGN_BRANCH_REGULAR_CARRYBACK_N",
        "Schedule B (Form 1116) foreign branch category: line 7, carryback to {0}",
    ),
    (
        "LINE_FOREIGN_BRANCH_REGULAR_DEDUCTION_YEAR_ABSORBED_N",
        "Schedule B (Form 1116) foreign branch category: line 7, absorbed in deduction year {0}",
    ),
    (
        "LINE_FOREIGN_BRANCH_REGULAR_NEW_CARRYFORWARD",
        "Schedule B (Form 1116) foreign branch category: line 8, new carryforward",
    ),
    (
        "LINE_FOREIGN_BRANCH_REGULAR_RETAINED_CARRYFORWARD",
        "Schedule B (Form 1116) foreign branch category: line 8, total carryforward retained",
    ),
    (
        "LINE_FOREIGN_BRANCH_TAX_REDUCTION",
        "Form 1116 foreign branch category: line 12, tax reduction",
    ),
    (
        "LINE_FOREIGN_BRANCH_TAXES_AVAILABLE",
        "Form 1116 foreign branch category: line 14, taxes available",
    ),
    (
        "LINE_FOREIGN_BRANCH_TAXES_FOR_CATEGORY",
        "Form 1116 foreign branch category: line 9, taxes for the category",
    ),
    ("LINE_FOREIGN_BRANCH_TAXES_TOTAL", "Form 1116 foreign branch category: line 8, foreign taxes"),
    (
        "LINE_FOREIGN_BRANCH_TAXES_WITH_CARRYOVER",
        "Form 1116 foreign branch category: line 11, taxes with carryover",
    ),
    (
        "LINE_FOREIGN_BRANCH_TOTAL_GROSS_INCOME",
        "Form 1116 foreign branch category, total column: line 1a, gross income",
    ),
    (
        "LINE_FOREIGN_BRANCH_TOTAL_TAXABLE_INCOME",
        "Form 1116 foreign branch category, total column: line 7, taxable income",
    ),
    (
        "LINE_FOREIGN_BRANCH_TOTAL_TOTAL_DEDUCTIONS",
        "Form 1116 foreign branch category, total column: line 6, total deductions",
    ),
    (
        "LINE_FOREIGN_BRANCH_UNUSED_TAX",
        "Schedule B (Form 1116) foreign branch category: line 6, excess foreign taxes",
    ),
    (
        "LINE_FOREIGN_BRANCH_US_INCOME_TAX",
        "Form 1116 foreign branch category: line 20, U.S. income tax",
    ),
    (
        "LINE_FOREIGN_BRANCH_WORLDWIDE_INCOME",
        "Form 1116 foreign branch category: line 18, worldwide income",
    ),
    ("LINE_FOREIGN_CREDIT_BOYCOTT", "Form 1116 line 34, international boycott reduction"),
    ("LINE_FOREIGN_CREDIT_BRANCH", "Form 1116 line 26, foreign branch category"),
    ("LINE_FOREIGN_CREDIT_GENERAL", "Form 1116 line 28, general category"),
    ("LINE_FOREIGN_CREDIT_LIMITED", "Form 1116 line 33, credit limited by U.S. tax"),
    ("LINE_FOREIGN_CREDIT_LUMP_SUM", "Form 1116 line 31, lump-sum distributions"),
    ("LINE_FOREIGN_CREDIT_PASSIVE", "Form 1116 line 27, passive category"),
    ("LINE_FOREIGN_CREDIT_SANCTIONED", "Form 1116 line 29, section 901(j) income"),
    ("LINE_FOREIGN_CREDIT_SECTION_951A", "Form 1116 line 25, section 951A category"),
    ("LINE_FOREIGN_CREDIT_SUM", "Form 1116 line 32, sum of categories"),
    ("LINE_FOREIGN_CREDIT_TREATY", "Form 1116 line 30, income re-sourced by treaty"),
    (
        "LINE_FOREIGN_GENERAL_N_APPORTIONED_DEDUCTIONS",
        "Form 1116 general category, {0}: line 3g, apportioned deductions",
    ),
    (
        "LINE_FOREIGN_GENERAL_N_DEDUCTIONS_SUBTOTAL",
        "Form 1116 general category, {0}: line 3c, deductions subtotal",
    ),
    (
        "LINE_FOREIGN_GENERAL_N_FOREIGN_LOSSES",
        "Form 1116 general category, {0}: line 5, foreign losses",
    ),
    (
        "LINE_FOREIGN_GENERAL_N_GROSS_FOREIGN_INCOME",
        "Form 1116 general category, {0}: line 3d, gross foreign income",
    ),
    (
        "LINE_FOREIGN_GENERAL_N_GROSS_INCOME",
        "Form 1116 general category, {0}: line 1a, gross income",
    ),
    (
        "LINE_FOREIGN_GENERAL_N_GROSS_INCOME_ALL_SOURCES",
        "Form 1116 general category, {0}: line 3e, gross income from all sources",
    ),
    (
        "LINE_FOREIGN_GENERAL_N_GROSS_INCOME_RATIO",
        "Form 1116 general category, {0}: line 3f, ratio of foreign to all income",
    ),
    (
        "LINE_FOREIGN_GENERAL_N_ITEMIZED_OR_STANDARD",
        "Form 1116 general category, {0}: line 3a, itemized or standard deduction",
    ),
    (
        "LINE_FOREIGN_GENERAL_N_MORTGAGE_INTEREST",
        "Form 1116 general category, {0}: line 4a, home mortgage interest",
    ),
    (
        "LINE_FOREIGN_GENERAL_N_OTHER_DEDUCTIONS",
        "Form 1116 general category, {0}: line 3b, other deductions",
    ),
    (
        "LINE_FOREIGN_GENERAL_N_OTHER_INTEREST",
        "Form 1116 general category, {0}: line 4b, other interest",
    ),
    (
        "LINE_FOREIGN_GENERAL_N_RELATED_EXPENSES",
        "Form 1116 general category, {0}: line 2, definitely related expenses",
    ),
    (
        "LINE_FOREIGN_GENERAL_N_TAXABLE_INCOME",
        "Form 1116 general category, {0}: line 7, taxable income",
    ),
    ("LINE_FOREIGN_GENERAL_N_TAXES", "Form 1116 general category, {0}: Part II foreign taxes"),
    (
        "LINE_FOREIGN_GENERAL_N_TOTAL_DEDUCTIONS",
        "Form 1116 general category, {0}: line 6, total deductions",
    ),
    (
        "LINE_FOREIGN_GENERAL_AMT_N_CARRY_ADJUSTED",
        "Schedule B (Form 1116) AMT general category, {0} column: line 3, adjusted carryover",
    ),
    (
        "LINE_FOREIGN_GENERAL_AMT_N_CARRY_EXPIRED",
        "Schedule B (Form 1116) AMT general category, {0} column: line 5, carryover expired",
    ),
    (
        "LINE_FOREIGN_GENERAL_AMT_N_CARRY_OPENING",
        "Schedule B (Form 1116) AMT general category, {0} column: line 1, prior carryover",
    ),
    (
        "LINE_FOREIGN_GENERAL_AMT_N_CARRYBACK_ADJUSTMENT",
        "Schedule B (Form 1116) AMT general category, {0} column: line 2a, carryback adjustment",
    ),
    (
        "LINE_FOREIGN_GENERAL_AMT_N_OTHER_ADJUSTMENTS",
        "Schedule B (Form 1116) AMT general category, {0} column: line 2c to 2g, other adjustments",
    ),
    (
        "LINE_FOREIGN_GENERAL_AMT_N_REDETERMINATION",
        "Schedule B (Form 1116) AMT general category, {0} column: line 2b, section 905(c) redeterminations",
    ),
    (
        "LINE_FOREIGN_GENERAL_AMT_CARRY_ADJUSTED_TOTAL",
        "Schedule B (Form 1116) AMT general category: line 3, total adjusted carryover",
    ),
    (
        "LINE_FOREIGN_GENERAL_AMT_CARRY_OPENING_TOTAL",
        "Schedule B (Form 1116) AMT general category: line 1, total prior carryover",
    ),
    ("LINE_FOREIGN_GENERAL_CARRYOVER", "Form 1116 general category: line 10, carryover"),
    (
        "LINE_FOREIGN_GENERAL_CATEGORY_CREDIT",
        "Form 1116 general category: line 24, category credit",
    ),
    (
        "LINE_FOREIGN_GENERAL_CREDIT_LIMITATION",
        "Form 1116 general category: line 23, credit limitation",
    ),
    ("LINE_FOREIGN_GENERAL_FOREIGN_INCOME", "Form 1116 general category: line 15, foreign income"),
    (
        "LINE_FOREIGN_GENERAL_HIGH_TAX_KICKOUT",
        "Form 1116 general category: line 13, high-tax kickout",
    ),
    (
        "LINE_FOREIGN_GENERAL_HTKO_GROSS_INCOME",
        "Form 1116 general category, high-taxed income column: line 1a, gross income",
    ),
    (
        "LINE_FOREIGN_GENERAL_HTKO_TAXABLE_INCOME",
        "Form 1116 general category, high-taxed income column: line 7, taxable income",
    ),
    (
        "LINE_FOREIGN_GENERAL_HTKO_TOTAL_DEDUCTIONS",
        "Form 1116 general category, high-taxed income column: line 6, total deductions",
    ),
    (
        "LINE_FOREIGN_GENERAL_INCOME_ADJUSTMENTS",
        "Form 1116 general category: line 16, income adjustments",
    ),
    (
        "LINE_FOREIGN_GENERAL_LIMITATION_INCREASE",
        "Form 1116 general category: line 22, limitation increase",
    ),
    (
        "LINE_FOREIGN_GENERAL_LIMITATION_RATIO",
        "Form 1116 general category: line 19, limitation ratio",
    ),
    ("LINE_FOREIGN_GENERAL_MAXIMUM_CREDIT", "Form 1116 general category: line 21, maximum credit"),
    (
        "LINE_FOREIGN_GENERAL_NET_FOREIGN_INCOME",
        "Form 1116 general category: line 17, net foreign income",
    ),
    (
        "LINE_FOREIGN_GENERAL_REGULAR_N_CARRY_ADJUSTED",
        "Schedule B (Form 1116) general category, {0} column: line 3, adjusted carryover",
    ),
    (
        "LINE_FOREIGN_GENERAL_REGULAR_N_CARRY_EXPIRED",
        "Schedule B (Form 1116) general category, {0} column: line 5, carryover expired",
    ),
    (
        "LINE_FOREIGN_GENERAL_REGULAR_N_CARRY_OPENING",
        "Schedule B (Form 1116) general category, {0} column: line 1, prior carryover",
    ),
    (
        "LINE_FOREIGN_GENERAL_REGULAR_N_CARRYBACK_ADJUSTMENT",
        "Schedule B (Form 1116) general category, {0} column: line 2a, carryback adjustment",
    ),
    (
        "LINE_FOREIGN_GENERAL_REGULAR_N_OTHER_ADJUSTMENTS",
        "Schedule B (Form 1116) general category, {0} column: line 2c to 2g, other adjustments",
    ),
    (
        "LINE_FOREIGN_GENERAL_REGULAR_N_REDETERMINATION",
        "Schedule B (Form 1116) general category, {0} column: line 2b, section 905(c) redeterminations",
    ),
    (
        "LINE_FOREIGN_GENERAL_REGULAR_CARRY_ADJUSTED_TOTAL",
        "Schedule B (Form 1116) general category: line 3, total adjusted carryover",
    ),
    (
        "LINE_FOREIGN_GENERAL_REGULAR_CARRY_OPENING_TOTAL",
        "Schedule B (Form 1116) general category: line 1, total prior carryover",
    ),
    ("LINE_FOREIGN_GENERAL_TAX_REDUCTION", "Form 1116 general category: line 12, tax reduction"),
    (
        "LINE_FOREIGN_GENERAL_TAXES_AVAILABLE",
        "Form 1116 general category: line 14, taxes available",
    ),
    (
        "LINE_FOREIGN_GENERAL_TAXES_FOR_CATEGORY",
        "Form 1116 general category: line 9, taxes for the category",
    ),
    ("LINE_FOREIGN_GENERAL_TAXES_TOTAL", "Form 1116 general category: line 8, foreign taxes"),
    (
        "LINE_FOREIGN_GENERAL_TAXES_WITH_CARRYOVER",
        "Form 1116 general category: line 11, taxes with carryover",
    ),
    (
        "LINE_FOREIGN_GENERAL_TOTAL_GROSS_INCOME",
        "Form 1116 general category, total column: line 1a, gross income",
    ),
    (
        "LINE_FOREIGN_GENERAL_TOTAL_TAXABLE_INCOME",
        "Form 1116 general category, total column: line 7, taxable income",
    ),
    (
        "LINE_FOREIGN_GENERAL_TOTAL_TOTAL_DEDUCTIONS",
        "Form 1116 general category, total column: line 6, total deductions",
    ),
    ("LINE_FOREIGN_GENERAL_US_INCOME_TAX", "Form 1116 general category: line 20, U.S. income tax"),
    (
        "LINE_FOREIGN_GENERAL_WORLDWIDE_INCOME",
        "Form 1116 general category: line 18, worldwide income",
    ),
    (
        "LINE_FOREIGN_PASSIVE_N_APPORTIONED_DEDUCTIONS",
        "Form 1116 passive category, {0}: line 3g, apportioned deductions",
    ),
    (
        "LINE_FOREIGN_PASSIVE_N_DEDUCTIONS_SUBTOTAL",
        "Form 1116 passive category, {0}: line 3c, deductions subtotal",
    ),
    (
        "LINE_FOREIGN_PASSIVE_N_FOREIGN_LOSSES",
        "Form 1116 passive category, {0}: line 5, foreign losses",
    ),
    (
        "LINE_FOREIGN_PASSIVE_N_GROSS_FOREIGN_INCOME",
        "Form 1116 passive category, {0}: line 3d, gross foreign income",
    ),
    (
        "LINE_FOREIGN_PASSIVE_N_GROSS_INCOME",
        "Form 1116 passive category, {0}: line 1a, gross income",
    ),
    (
        "LINE_FOREIGN_PASSIVE_N_GROSS_INCOME_ALL_SOURCES",
        "Form 1116 passive category, {0}: line 3e, gross income from all sources",
    ),
    (
        "LINE_FOREIGN_PASSIVE_N_GROSS_INCOME_RATIO",
        "Form 1116 passive category, {0}: line 3f, ratio of foreign to all income",
    ),
    (
        "LINE_FOREIGN_PASSIVE_N_ITEMIZED_OR_STANDARD",
        "Form 1116 passive category, {0}: line 3a, itemized or standard deduction",
    ),
    (
        "LINE_FOREIGN_PASSIVE_N_MORTGAGE_INTEREST",
        "Form 1116 passive category, {0}: line 4a, home mortgage interest",
    ),
    (
        "LINE_FOREIGN_PASSIVE_N_OTHER_DEDUCTIONS",
        "Form 1116 passive category, {0}: line 3b, other deductions",
    ),
    (
        "LINE_FOREIGN_PASSIVE_N_OTHER_INTEREST",
        "Form 1116 passive category, {0}: line 4b, other interest",
    ),
    (
        "LINE_FOREIGN_PASSIVE_N_RELATED_EXPENSES",
        "Form 1116 passive category, {0}: line 2, definitely related expenses",
    ),
    (
        "LINE_FOREIGN_PASSIVE_N_TAXABLE_INCOME",
        "Form 1116 passive category, {0}: line 7, taxable income",
    ),
    ("LINE_FOREIGN_PASSIVE_N_TAXES", "Form 1116 passive category, {0}: Part II foreign taxes"),
    (
        "LINE_FOREIGN_PASSIVE_N_TOTAL_DEDUCTIONS",
        "Form 1116 passive category, {0}: line 6, total deductions",
    ),
    (
        "LINE_FOREIGN_PASSIVE_AMT_N_CARRY_ADJUSTED",
        "Schedule B (Form 1116) AMT passive category, {0} column: line 3, adjusted carryover",
    ),
    (
        "LINE_FOREIGN_PASSIVE_AMT_N_CARRY_EXPIRED",
        "Schedule B (Form 1116) AMT passive category, {0} column: line 5, carryover expired",
    ),
    (
        "LINE_FOREIGN_PASSIVE_AMT_N_CARRY_OPENING",
        "Schedule B (Form 1116) AMT passive category, {0} column: line 1, prior carryover",
    ),
    (
        "LINE_FOREIGN_PASSIVE_AMT_N_CARRYBACK_ADJUSTMENT",
        "Schedule B (Form 1116) AMT passive category, {0} column: line 2a, carryback adjustment",
    ),
    (
        "LINE_FOREIGN_PASSIVE_AMT_N_OTHER_ADJUSTMENTS",
        "Schedule B (Form 1116) AMT passive category, {0} column: line 2c to 2g, other adjustments",
    ),
    (
        "LINE_FOREIGN_PASSIVE_AMT_N_REDETERMINATION",
        "Schedule B (Form 1116) AMT passive category, {0} column: line 2b, section 905(c) redeterminations",
    ),
    (
        "LINE_FOREIGN_PASSIVE_AMT_CARRY_ADJUSTED_TOTAL",
        "Schedule B (Form 1116) AMT passive category: line 3, total adjusted carryover",
    ),
    (
        "LINE_FOREIGN_PASSIVE_AMT_CARRY_OPENING_TOTAL",
        "Schedule B (Form 1116) AMT passive category: line 1, total prior carryover",
    ),
    ("LINE_FOREIGN_PASSIVE_CARRYOVER", "Form 1116 passive category: line 10, carryover"),
    (
        "LINE_FOREIGN_PASSIVE_CATEGORY_CREDIT",
        "Form 1116 passive category: line 24, category credit",
    ),
    (
        "LINE_FOREIGN_PASSIVE_CREDIT_LIMITATION",
        "Form 1116 passive category: line 23, credit limitation",
    ),
    ("LINE_FOREIGN_PASSIVE_FOREIGN_INCOME", "Form 1116 passive category: line 15, foreign income"),
    (
        "LINE_FOREIGN_PASSIVE_HIGH_TAX_KICKOUT",
        "Form 1116 passive category: line 13, high-tax kickout",
    ),
    (
        "LINE_FOREIGN_PASSIVE_HTKO_GROSS_INCOME",
        "Form 1116 passive category, high-taxed income column: line 1a, gross income",
    ),
    (
        "LINE_FOREIGN_PASSIVE_HTKO_TAXABLE_INCOME",
        "Form 1116 passive category, high-taxed income column: line 7, taxable income",
    ),
    (
        "LINE_FOREIGN_PASSIVE_HTKO_TOTAL_DEDUCTIONS",
        "Form 1116 passive category, high-taxed income column: line 6, total deductions",
    ),
    (
        "LINE_FOREIGN_PASSIVE_INCOME_ADJUSTMENTS",
        "Form 1116 passive category: line 16, income adjustments",
    ),
    (
        "LINE_FOREIGN_PASSIVE_LIMITATION_INCREASE",
        "Form 1116 passive category: line 22, limitation increase",
    ),
    (
        "LINE_FOREIGN_PASSIVE_LIMITATION_RATIO",
        "Form 1116 passive category: line 19, limitation ratio",
    ),
    ("LINE_FOREIGN_PASSIVE_MAXIMUM_CREDIT", "Form 1116 passive category: line 21, maximum credit"),
    (
        "LINE_FOREIGN_PASSIVE_NET_FOREIGN_INCOME",
        "Form 1116 passive category: line 17, net foreign income",
    ),
    (
        "LINE_FOREIGN_PASSIVE_REGULAR_N_CARRY_ADJUSTED",
        "Schedule B (Form 1116) passive category, {0} column: line 3, adjusted carryover",
    ),
    (
        "LINE_FOREIGN_PASSIVE_REGULAR_N_CARRY_EXPIRED",
        "Schedule B (Form 1116) passive category, {0} column: line 5, carryover expired",
    ),
    (
        "LINE_FOREIGN_PASSIVE_REGULAR_N_CARRY_OPENING",
        "Schedule B (Form 1116) passive category, {0} column: line 1, prior carryover",
    ),
    (
        "LINE_FOREIGN_PASSIVE_REGULAR_N_CARRYBACK_ADJUSTMENT",
        "Schedule B (Form 1116) passive category, {0} column: line 2a, carryback adjustment",
    ),
    (
        "LINE_FOREIGN_PASSIVE_REGULAR_N_OTHER_ADJUSTMENTS",
        "Schedule B (Form 1116) passive category, {0} column: line 2c to 2g, other adjustments",
    ),
    (
        "LINE_FOREIGN_PASSIVE_REGULAR_N_REDETERMINATION",
        "Schedule B (Form 1116) passive category, {0} column: line 2b, section 905(c) redeterminations",
    ),
    (
        "LINE_FOREIGN_PASSIVE_REGULAR_CARRY_ADJUSTED_TOTAL",
        "Schedule B (Form 1116) passive category: line 3, total adjusted carryover",
    ),
    (
        "LINE_FOREIGN_PASSIVE_REGULAR_CARRY_OPENING_TOTAL",
        "Schedule B (Form 1116) passive category: line 1, total prior carryover",
    ),
    ("LINE_FOREIGN_PASSIVE_TAX_REDUCTION", "Form 1116 passive category: line 12, tax reduction"),
    (
        "LINE_FOREIGN_PASSIVE_TAXES_AVAILABLE",
        "Form 1116 passive category: line 14, taxes available",
    ),
    (
        "LINE_FOREIGN_PASSIVE_TAXES_FOR_CATEGORY",
        "Form 1116 passive category: line 9, taxes for the category",
    ),
    ("LINE_FOREIGN_PASSIVE_TAXES_TOTAL", "Form 1116 passive category: line 8, foreign taxes"),
    (
        "LINE_FOREIGN_PASSIVE_TAXES_WITH_CARRYOVER",
        "Form 1116 passive category: line 11, taxes with carryover",
    ),
    (
        "LINE_FOREIGN_PASSIVE_TOTAL_GROSS_INCOME",
        "Form 1116 passive category, total column: line 1a, gross income",
    ),
    (
        "LINE_FOREIGN_PASSIVE_TOTAL_TAXABLE_INCOME",
        "Form 1116 passive category, total column: line 7, taxable income",
    ),
    (
        "LINE_FOREIGN_PASSIVE_TOTAL_TOTAL_DEDUCTIONS",
        "Form 1116 passive category, total column: line 6, total deductions",
    ),
    ("LINE_FOREIGN_PASSIVE_US_INCOME_TAX", "Form 1116 passive category: line 20, U.S. income tax"),
    (
        "LINE_FOREIGN_PASSIVE_WORLDWIDE_INCOME",
        "Form 1116 passive category: line 18, worldwide income",
    ),
    (
        "LINE_FOREIGN_REGULAR_BRANCH_OVERALL_DOMESTIC_LOSS_NEW",
        "Foreign regular branch overall domestic loss new",
    ),
    (
        "LINE_FOREIGN_REGULAR_BRANCH_OVERALL_DOMESTIC_LOSS_RECAPTURED",
        "Foreign regular branch overall domestic loss recaptured",
    ),
    (
        "LINE_FOREIGN_REGULAR_BRANCH_OVERALL_DOMESTIC_LOSS_REMAINING",
        "Foreign regular branch overall domestic loss remaining",
    ),
    (
        "LINE_FOREIGN_REGULAR_BRANCH_OVERALL_FOREIGN_LOSS_NEW",
        "Foreign regular branch overall foreign loss new",
    ),
    (
        "LINE_FOREIGN_REGULAR_BRANCH_OVERALL_FOREIGN_LOSS_RECAPTURED",
        "Foreign regular branch overall foreign loss recaptured",
    ),
    (
        "LINE_FOREIGN_REGULAR_BRANCH_OVERALL_FOREIGN_LOSS_REMAINING",
        "Foreign regular branch overall foreign loss remaining",
    ),
    (
        "LINE_FOREIGN_REGULAR_BRANCH_SEPARATE_LOSS_NETTED",
        "Foreign regular branch separate loss netted",
    ),
    ("LINE_FOREIGN_REGULAR_BRANCH_SEPARATE_LOSS_NEW", "Foreign regular branch separate loss new"),
    (
        "LINE_FOREIGN_REGULAR_BRANCH_SEPARATE_LOSS_RECAPTURED",
        "Foreign regular branch separate loss recaptured",
    ),
    (
        "LINE_FOREIGN_REGULAR_BRANCH_SEPARATE_LOSS_REMAINING",
        "Foreign regular branch separate loss remaining",
    ),
    (
        "LINE_FOREIGN_REGULAR_SANCTIONED_OVERALL_DOMESTIC_LOSS_NEW",
        "Foreign regular sanctioned overall domestic loss new",
    ),
    (
        "LINE_FOREIGN_REGULAR_SANCTIONED_OVERALL_FOREIGN_LOSS_NEW",
        "Foreign regular sanctioned overall foreign loss new",
    ),
    (
        "LINE_FOREIGN_REGULAR_SANCTIONED_SEPARATE_LOSS_NEW",
        "Foreign regular sanctioned separate loss new",
    ),
    (
        "LINE_FOREIGN_SANCTIONED_N_APPORTIONED_DEDUCTIONS",
        "Form 1116 section 901(j) category, {0}: line 3g, apportioned deductions",
    ),
    (
        "LINE_FOREIGN_SANCTIONED_N_DEDUCTIONS_SUBTOTAL",
        "Form 1116 section 901(j) category, {0}: line 3c, deductions subtotal",
    ),
    (
        "LINE_FOREIGN_SANCTIONED_N_FOREIGN_LOSSES",
        "Form 1116 section 901(j) category, {0}: line 5, foreign losses",
    ),
    (
        "LINE_FOREIGN_SANCTIONED_N_GROSS_FOREIGN_INCOME",
        "Form 1116 section 901(j) category, {0}: line 3d, gross foreign income",
    ),
    (
        "LINE_FOREIGN_SANCTIONED_N_GROSS_INCOME",
        "Form 1116 section 901(j) category, {0}: line 1a, gross income",
    ),
    (
        "LINE_FOREIGN_SANCTIONED_N_GROSS_INCOME_ALL_SOURCES",
        "Form 1116 section 901(j) category, {0}: line 3e, gross income from all sources",
    ),
    (
        "LINE_FOREIGN_SANCTIONED_N_GROSS_INCOME_RATIO",
        "Form 1116 section 901(j) category, {0}: line 3f, ratio of foreign to all income",
    ),
    (
        "LINE_FOREIGN_SANCTIONED_N_ITEMIZED_OR_STANDARD",
        "Form 1116 section 901(j) category, {0}: line 3a, itemized or standard deduction",
    ),
    (
        "LINE_FOREIGN_SANCTIONED_N_MORTGAGE_INTEREST",
        "Form 1116 section 901(j) category, {0}: line 4a, home mortgage interest",
    ),
    (
        "LINE_FOREIGN_SANCTIONED_N_OTHER_DEDUCTIONS",
        "Form 1116 section 901(j) category, {0}: line 3b, other deductions",
    ),
    (
        "LINE_FOREIGN_SANCTIONED_N_OTHER_INTEREST",
        "Form 1116 section 901(j) category, {0}: line 4b, other interest",
    ),
    (
        "LINE_FOREIGN_SANCTIONED_N_RELATED_EXPENSES",
        "Form 1116 section 901(j) category, {0}: line 2, definitely related expenses",
    ),
    (
        "LINE_FOREIGN_SANCTIONED_N_TAXABLE_INCOME",
        "Form 1116 section 901(j) category, {0}: line 7, taxable income",
    ),
    (
        "LINE_FOREIGN_SANCTIONED_N_TAXES",
        "Form 1116 section 901(j) category, {0}: Part II foreign taxes",
    ),
    (
        "LINE_FOREIGN_SANCTIONED_N_TOTAL_DEDUCTIONS",
        "Form 1116 section 901(j) category, {0}: line 6, total deductions",
    ),
    (
        "LINE_FOREIGN_SANCTIONED_FOREIGN_INCOME",
        "Form 1116 section 901(j) category: line 15, foreign income",
    ),
    (
        "LINE_FOREIGN_SANCTIONED_HTKO_GROSS_INCOME",
        "Form 1116 section 901(j) category, high-taxed income column: line 1a, gross income",
    ),
    (
        "LINE_FOREIGN_SANCTIONED_HTKO_TAXABLE_INCOME",
        "Form 1116 section 901(j) category, high-taxed income column: line 7, taxable income",
    ),
    (
        "LINE_FOREIGN_SANCTIONED_HTKO_TOTAL_DEDUCTIONS",
        "Form 1116 section 901(j) category, high-taxed income column: line 6, total deductions",
    ),
    (
        "LINE_FOREIGN_SANCTIONED_INCOME_ADJUSTMENTS",
        "Form 1116 section 901(j) category: line 16, income adjustments",
    ),
    (
        "LINE_FOREIGN_SANCTIONED_NET_FOREIGN_INCOME",
        "Form 1116 section 901(j) category: line 17, net foreign income",
    ),
    (
        "LINE_FOREIGN_SANCTIONED_TAXES_TOTAL",
        "Form 1116 section 901(j) category: line 8, foreign taxes",
    ),
    (
        "LINE_FOREIGN_SANCTIONED_TOTAL_GROSS_INCOME",
        "Form 1116 section 901(j) category, total column: line 1a, gross income",
    ),
    (
        "LINE_FOREIGN_SANCTIONED_TOTAL_TAXABLE_INCOME",
        "Form 1116 section 901(j) category, total column: line 7, taxable income",
    ),
    (
        "LINE_FOREIGN_SANCTIONED_TOTAL_TOTAL_DEDUCTIONS",
        "Form 1116 section 901(j) category, total column: line 6, total deductions",
    ),
    ("LINE_FOREIGN_WORKSHEET_A_LINE_N", "Foreign capital Worksheet A line {0}"),
    ("LINE_FOREIGN_WORKSHEET_B_LINE_N", "Foreign capital Worksheet B line {0}"),
    (
        "EXPLAIN_FORM_1116_LINE_1A",
        "Gross income of this category from the country, after the Form 2555 exclusion and the qualified dividend and capital gain adjustments.",
    ),
    (
        "EXPLAIN_FORM_1116_LINE_2",
        "Expenses definitely related to this foreign income: its business expenses, state income tax on it, and the Schedule K-3 deductions of its column.",
    ),
    (
        "EXPLAIN_FORM_1116_LINE_3A",
        "Standard deduction, or medical, sales tax, real estate and personal property tax deductions not definitely related to any income, after the itemized deduction limit.",
    ),
    (
        "EXPLAIN_FORM_1116_LINE_3B",
        "Other deductions not definitely related to any income: the IRA deduction, other Schedule 1 adjustments, casualty losses and Schedule K-3 column (f) deductions.",
    ),
    ("EXPLAIN_FORM_1116_LINE_3C", "Lines 3a and 3b, the same in every column."),
    (
        "EXPLAIN_FORM_1116_LINE_3D",
        "Gross foreign income before any rate adjustment, including income excluded on Form 2555.",
    ),
    ("EXPLAIN_FORM_1116_LINE_3E", "Gross income from all sources, the same in every column."),
    ("EXPLAIN_FORM_1116_LINE_3F", "Line 3d divided by line 3e, to four places, at most 1."),
    ("EXPLAIN_FORM_1116_LINE_3G", "Line 3c times line 3f."),
    (
        "EXPLAIN_FORM_1116_LINE_4A",
        "Home mortgage interest by the Form 1116 worksheet, which leaves income excluded on Form 2555 out of both gross incomes.",
    ),
    (
        "EXPLAIN_FORM_1116_LINE_4B",
        "Other interest expense apportioned to this foreign income by assets or gross income.",
    ),
    ("EXPLAIN_FORM_1116_LINE_5", "Foreign capital losses after the Worksheet A or B adjustments."),
    ("EXPLAIN_FORM_1116_LINE_6", "Lines 2, 3g, 4a, 4b and 5."),
    ("EXPLAIN_FORM_1116_LINE_7", "Line 1a less line 6."),
    ("EXPLAIN_FORM_1116_PART_II_W", "Foreign tax paid or accrued to this country in U.S. dollars."),
    ("EXPLAIN_FORM_1116_LINE_8", "Foreign taxes paid or accrued, from Part II."),
    ("EXPLAIN_FORM_1116_LINE_9", "Line 8 for this category."),
    (
        "EXPLAIN_FORM_1116_LINE_10",
        "Unused foreign taxes carried to 2026 from Schedule B (Form 1116) line 3.",
    ),
    ("EXPLAIN_FORM_1116_LINE_11", "Lines 9 and 10."),
    (
        "EXPLAIN_FORM_1116_LINE_12",
        "Taxes on income excluded on Form 2555 and reductions from Schedule K-3, as a negative amount.",
    ),
    (
        "EXPLAIN_FORM_1116_LINE_13",
        "High-taxed passive income moved to the general category: its tax out of the passive category and into the general category.",
    ),
    ("EXPLAIN_FORM_1116_LINE_14", "Foreign taxes available for the credit: lines 11, 12 and 13."),
    (
        "EXPLAIN_FORM_1116_LINE_15",
        "Taxable income or loss from foreign sources, the total of line 7.",
    ),
    (
        "EXPLAIN_FORM_1116_LINE_16",
        "Adjustments for the section 461(l) loss and for foreign and domestic loss accounts, and other changes needed to reach line 17.",
    ),
    ("EXPLAIN_FORM_1116_LINE_17", "Net foreign source taxable income in this category."),
    (
        "EXPLAIN_FORM_1116_LINE_18",
        "Taxable income from all sources, with the worksheet adjustment for lower-rate qualified dividends and capital gains.",
    ),
    ("EXPLAIN_FORM_1116_LINE_19", "Line 17 divided by line 18, to four places, at most 1."),
    (
        "EXPLAIN_FORM_1116_LINE_20",
        "U.S. income tax against which the credit is allowed (the regular tax, or the tentative minimum tax for the alternative minimum tax).",
    ),
    ("EXPLAIN_FORM_1116_LINE_21", "Line 20 times line 19."),
    (
        "EXPLAIN_FORM_1116_LINE_22",
        "Increase in the limitation under section 960(c); not calculated.",
    ),
    ("EXPLAIN_FORM_1116_LINE_23", "The limitation on the credit for this category."),
    ("EXPLAIN_FORM_1116_LINE_24", "The smaller of line 14 and line 23."),
    ("EXPLAIN_FORM_1116_LINE_25", "Credit for section 951A category income; not calculated."),
    ("EXPLAIN_FORM_1116_LINE_26", "Credit for the foreign branch category."),
    ("EXPLAIN_FORM_1116_LINE_27", "Credit for the passive category."),
    ("EXPLAIN_FORM_1116_LINE_28", "Credit for the general category."),
    (
        "EXPLAIN_FORM_1116_LINE_29",
        "No credit is allowed for taxes paid to a section 901(j) country.",
    ),
    ("EXPLAIN_FORM_1116_LINE_30", "Credit for income re-sourced by treaty; not calculated."),
    ("EXPLAIN_FORM_1116_LINE_31", "Credit for lump-sum distributions; not calculated."),
    ("EXPLAIN_FORM_1116_LINE_32", "Lines 25 through 31."),
    ("EXPLAIN_FORM_1116_LINE_33", "The smaller of line 20 and line 32."),
    ("EXPLAIN_FORM_1116_LINE_34", "Reduction for an international boycott; not calculated."),
    (
        "EXPLAIN_SCHEDULE_B_LINE_1",
        "The carryover of this year from line 8 of the prior Schedule B.",
    ),
    (
        "EXPLAIN_SCHEDULE_B_LINE_2A",
        "Difference between the estimated and the actual carryback to 2024.",
    ),
    (
        "EXPLAIN_SCHEDULE_B_LINE_2B",
        "Change from a foreign tax redetermination reported on Schedule C (Form 1116).",
    ),
    ("EXPLAIN_SCHEDULE_B_LINE_2C", "Domestic audit adjustments and other changes."),
    ("EXPLAIN_SCHEDULE_B_LINE_3", "Lines 1 and 2a through 2g."),
    (
        "EXPLAIN_SCHEDULE_B_LINE_4",
        "Carryover used against the excess limitation, oldest year first.",
    ),
    ("EXPLAIN_SCHEDULE_B_LINE_5", "The remainder of the oldest year expires."),
    ("EXPLAIN_SCHEDULE_B_LINE_6", "Foreign taxes above the credit limitation for 2026."),
    ("EXPLAIN_SCHEDULE_B_LINE_7", "Excess foreign taxes carried back to 2025."),
    ("EXPLAIN_SCHEDULE_B_LINE_8", "Carryover left in this year's column."),
    ("EXPLAIN_SCHEDULE_B_LINE_8_RETURN_YEAR", "Excess foreign taxes carried forward from 2026."),
    ("EXPLAIN_SCHEDULE_B_LINE_1_TOTAL", "Total of the prior carryovers."),
    ("EXPLAIN_SCHEDULE_B_LINE_3_TOTAL", "Total adjusted carryover; it is Form 1116 line 10."),
    ("EXPLAIN_SCHEDULE_B_LINE_4_TOTAL", "Total carryover used in 2026."),
    ("EXPLAIN_SCHEDULE_B_LINE_5_TOTAL", "Total carryover that expired."),
    ("EXPLAIN_SCHEDULE_B_LINE_8_TOTAL", "Total carryover from earlier years left for later years."),
    (
        "EXPLAIN_FOREIGN_CAPITAL_WORKSHEET",
        "A line of the Form 1116 adjustment worksheet for foreign capital gains and losses; the worksheet is kept for your records.",
    ),
    (
        "WARN_FOREIGN_SITUATION_LEFT_OUT",
        "{0}: this estimate does not figure it, so no foreign tax credit is claimed; if one applies, your tax may be lower.",
    ),
    (
        "WARN_FTC_RELEASED_LOSS_ASSUMED",
        "{0}'s losses from earlier years allowed now are treated as foreign; your credit may be higher.",
    ),
    ("LINE_FOREIGN_INCOME_TAX_DEDUCTION", "Foreign income tax deduction"),
    ("LINE_FOREIGN_INVESTMENT_TAX_DEDUCTION", "Foreign investment tax deduction"),
    (
        "EXPLAIN_FOREIGN_INCOME_TAX_DEDUCTION",
        "Foreign income taxes deducted on Schedule A line 6, outside the state and local tax limit, after removing taxes on excluded foreign earnings and applying K-1 basis limits.",
    ),
    (
        "EXPLAIN_FOREIGN_INVESTMENT_TAX_DEDUCTION",
        "The deducted foreign income taxes attributable to net investment income join Form 8960 line 9b, subject to the overall allowed itemized-deduction limit.",
    ),
    (
        "WARN_FOREIGN_TAX_CREDIT_NOT_CLAIMED",
        "Foreign income that is not all passive income shown on payee statements can't use the small-credit election, so no foreign tax credit is claimed; choose the credit using Form 1116 to claim it.",
    ),
    (
        "WARN_FOREIGN_CREDIT_CARRYOVER_LEFT_OUT",
        "Foreign tax credit carryovers from a joint return with someone you are not filing jointly with now are left out of this estimate; your tax may be lower.",
    ),
    (
        "WARN_FOREIGN_LOSS_ACCOUNTS_LEFT_OUT",
        "Foreign tax credit loss accounts from a joint return with someone you are not filing jointly with now are left out of this estimate; your tax may be higher or lower.",
    ),
    ("FORM_FORM3468", "Form 3468"),
    ("LINE_SECTION_N_N_AGGREGATE", "Section {0} contracts {1} profit or loss"),
    (
        "EXPLAIN_SECTION_1256_N_AGGREGATE",
        "Form 1099-B box 11 aggregate profit or (loss) on regulated futures, foreign currency and section 1256 option contracts for this statement; Form 6781 line 1.",
    ),
    ("LINE_FORM_N_LOSSES", "Form {0} section 1256 losses"),
    ("EXPLAIN_FORM_6781_LOSSES", "Losses on Form 6781 line 1, column (b), added (line 2)."),
    ("LINE_FORM_N_GAINS", "Form {0} section 1256 gains"),
    ("EXPLAIN_FORM_6781_GAINS", "Gains on Form 6781 line 1, column (c), added (line 2)."),
    ("LINE_FORM_N_NET", "Form {0} net section 1256 gain or loss"),
    (
        "EXPLAIN_FORM_6781_NET",
        "Form 6781 lines 3 and 5: losses and gains combined. Line 4 adjustments (straddle and hedging cases) are refused or left out, so none applies.",
    ),
    ("LINE_FORM_N_CARRYBACK", "Form {0} net section 1256 loss carried back"),
    (
        "EXPLAIN_FORM_6781_CARRYBACK",
        "Form 6781 line 6: the net section 1256 contracts loss you elected to carry back to 2023–2025 (box D). It is treated as 40% short-term and 60% long-term gain of this year (section 1212(c)(6)(A)); the refund is not in the estimate.",
    ),
    ("LINE_FORM_N_TOTAL", "Form {0} section 1256 gain or loss after carryback"),
    ("EXPLAIN_FORM_6781_TOTAL", "Form 6781 line 7: line 5 plus line 6, before the 40/60 split."),
    ("LINE_FORM_N_SHORT_TERM", "Form {0} short-term section 1256 gain or loss"),
    (
        "EXPLAIN_FORM_6781_SHORT_TERM",
        "Form 6781 line 8: 40% of line 7, short-term whatever the holding period (section 1256(a)(3)); Schedule D line 4.",
    ),
    ("LINE_FORM_N_LONG_TERM", "Form {0} long-term section 1256 gain or loss"),
    (
        "EXPLAIN_FORM_6781_LONG_TERM",
        "Form 6781 line 9: 60% of line 7, long-term whatever the holding period (section 1256(a)(3)); Schedule D line 11.",
    ),
    ("LINE_SECTION_N_NET_LOSS", "Section {0} net contracts loss limit"),
    (
        "EXPLAIN_SECTION_1256_NET_LOSS",
        "The most a carryback election can carry back under Form 6781 box D: the lesser of the loss that would carry over to 2027 without a carryback and the Form 6781 line 5 loss plus any 2025 capital loss carryover, less the $3,000 ($1,500 if married filing separately) limit. An upper bound when a 2025 carryover came from section 1256 contracts.",
    ),
    (
        "EXPLAIN_FORM8582CR_LINE_3A",
        "Form 8582-CR line 3a: 2026 low-income housing credits from property placed in service after 1989 (Worksheet 3, column (a)).",
    ),
    (
        "EXPLAIN_FORM8582CR_LINE_3B",
        "Form 8582-CR line 3b: prior-year unallowed low-income housing credits (Worksheet 3, column (b)).",
    ),
    ("EXPLAIN_FORM8582CR_LINE_3C", "Form 8582-CR line 3c: lines 3a and 3b."),
    (
        "EXPLAIN_FORM8582CR_LINE_4A",
        "Form 8582-CR line 4a: 2026 passive credits other than rental real estate, rehabilitation and low-income housing credits (Worksheet 4, column (a)).",
    ),
    (
        "EXPLAIN_FORM8582CR_LINE_4B",
        "Form 8582-CR line 4b: prior-year unallowed credits of those other passive activities (Worksheet 4, column (b)).",
    ),
    ("EXPLAIN_FORM8582CR_LINE_4C", "Form 8582-CR line 4c: lines 4a and 4b."),
    (
        "EXPLAIN_FORM8582CR_LINE_5",
        "Form 8582-CR line 5: the passive activity credits of lines 2c, 3c and 4c (line 1c is zero).",
    ),
    (
        "EXPLAIN_FORM8582CR_LINE_6",
        "Form 8582-CR line 6: the regular tax on your taxable income less the regular tax without your net passive income.",
    ),
    (
        "EXPLAIN_FORM8582CR_LINE_7",
        "Form 8582-CR line 7: line 5 less line 6. At zero every passive credit is allowed.",
    ),
    (
        "EXPLAIN_FORM8582CR_LINE_31",
        "Form 8582-CR line 31: line 19, line 7 less line 16 (Part II is zero).",
    ),
    (
        "EXPLAIN_FORM8582CR_LINE_33",
        "Form 8582-CR line 33: line 31 less line 32, the rehabilitation allowance of line 30.",
    ),
    ("EXPLAIN_FORM8582CR_LINE_34", "Form 8582-CR line 34: the smaller of line 3c and line 33."),
    (
        "EXPLAIN_FORM8582CR_LINE_35",
        "Form 8582-CR line 35: the regular tax on your taxable income less the tax on it reduced by the special allowance that remains, $25,000 ($12,500 married filing separately and living apart) less the rental losses Form 8582 line 9 allowed, less lines 16 and 30. No income phase-out applies to low-income housing credits (section 469(i)(3)(C)).",
    ),
    (
        "EXPLAIN_FORM8582CR_LINE_36",
        "Form 8582-CR line 36: the smaller of line 34 and line 35. It is zero for a married person filing separately who lived with a spouse during the year.",
    ),
    (
        "EXPLAIN_FORM8582CR_LINE_37",
        "Form 8582-CR line 37: passive activity credits allowed for 2026, lines 6, 30 and 36 (line 16 is zero). Form 3800 takes it on Part I line 3 and Part II line 33; the rest carries to 2027.",
    ),
    (
        "EXPLAIN_FORM8582_LINE_1A",
        "Form 8582 line 1a: the total of Part IV column (a), the net income of rental real estate activities with active participation.",
    ),
    (
        "EXPLAIN_FORM8582_LINE_1B",
        "Form 8582 line 1b: the total of Part IV column (b), the net loss of rental real estate activities with active participation.",
    ),
    (
        "EXPLAIN_FORM8582_LINE_1C",
        "Form 8582 line 1c: the total of Part IV column (c), earlier years' unallowed losses of activities in which you actively participated in the year the loss arose and this year.",
    ),
    (
        "EXPLAIN_FORM8582_LINE_2A",
        "Form 8582 line 2a: the total of Part V column (a), the net income of all other passive activities.",
    ),
    (
        "EXPLAIN_FORM8582_LINE_2B",
        "Form 8582 line 2b: the total of Part V column (b), the net loss of all other passive activities.",
    ),
    (
        "EXPLAIN_FORM8582_LINE_2C",
        "Form 8582 line 2c: the total of Part V column (c), earlier years' unallowed losses of all other passive activities, including a rental loss from a year without active participation.",
    ),
    (
        "EXPLAIN_FORM8582_PART_IX_N_ORDINARY_UNALLOWED",
        "Form 8582 Part IX column (d): the part of this activity's ordinary loss that the passive activity rules do not allow.",
    ),
    (
        "EXPLAIN_FORM8582_PART_IX_N_ORDINARY_ALLOWED",
        "Form 8582 Part IX column (e): the part of this activity's ordinary loss that the passive activity rules allow.",
    ),
    (
        "EXPLAIN_FORM8582_PART_IX_N_SECTION1231_UNALLOWED",
        "Form 8582 Part IX column (d): the part of this activity's section 1231 loss that the passive activity rules do not allow.",
    ),
    (
        "EXPLAIN_FORM8582_PART_IX_N_SECTION1231_ALLOWED",
        "Form 8582 Part IX column (e): the part of this activity's section 1231 loss that the passive activity rules allow.",
    ),
    (
        "EXPLAIN_FORM8582_LINE_1D",
        "Form 8582 line 1d: the net income or loss of rental real estate activities with active participation, with prior-year unallowed losses.",
    ),
    (
        "EXPLAIN_FORM8582_LINE_2D",
        "Form 8582 line 2d: the net income or loss of all other passive activities, with prior-year unallowed losses.",
    ),
    (
        "EXPLAIN_FORM8582_LINE_3",
        "Form 8582 line 3: lines 1d and 2d combined. At zero or more, every passive loss is allowed.",
    ),
    (
        "EXPLAIN_FORM8582_LINE_4",
        "Form 8582 line 4: the smaller of the loss on line 1d and the loss on line 3.",
    ),
    ("EXPLAIN_FORM8582_LINE_5", "Form 8582 line 5: $150,000 ($75,000 married filing separately)."),
    ("EXPLAIN_FORM8582_LINE_6", "Form 8582 line 6: your modified adjusted gross income."),
    ("EXPLAIN_FORM8582_LINE_7", "Form 8582 line 7: line 5 less line 6."),
    (
        "EXPLAIN_FORM8582_LINE_8",
        "Form 8582 line 8: half of line 7, at most $25,000 ($12,500 married filing separately and living apart all year; nothing when living together).",
    ),
    ("EXPLAIN_FORM8582_LINE_10", "Form 8582 line 10: the income, if any, on lines 1a and 2a."),
    (
        "EXPLAIN_FORM8582_LINE_11",
        "Form 8582 line 11: the total losses allowed from all passive activities, lines 9 and 10.",
    ),
    (
        "EXPLAIN_FORM8582_LINE_9",
        "Form 8582 line 9: the smaller of line 4 and line 8, the rental losses the special allowance lets you deduct.",
    ),
    ("LINE_ITEMIZED_MORTGAGE_OTHER_USE_BOX", "Mortgage proceeds used for other purposes"),
    ("LINE_ITEMIZED_BELOW_STANDARD_BOX", "Election to itemize below the standard deduction"),
    ("LINE_SCHEDULE_B_FOREIGN_ACCOUNTS", "Foreign financial account"),
    ("LINE_SCHEDULE_B_FBAR_REQUIRED", "FBAR filing requirement"),
    ("LINE_SCHEDULE_B_FOREIGN_TRUST", "Foreign trust transaction"),
    ("LINE_SCHEDULE_B_COUNTRY_N", "Foreign-account country {0}"),
    ("EXPLAIN_PRINT_ANSWER", "{0}"),
    ("LINE_FARM_N_HEADER_ACTIVITY_CODE", "Farm {0} principal agricultural activity code"),
    ("LINE_FARM_N_HEADER_EIN", "Farm {0} employer identification number"),
    ("LINE_FARM_N_HEADER_INFORMATION_REQUIRED", "Farm {0} information returns required"),
    ("LINE_FARM_N_HEADER_INFORMATION_FILED", "Farm {0} information returns filed or to be filed"),
    ("PRINT_UNKNOWN", "Not sure"),
    // Calculation labels (line_label): each calculation id pattern, its
    // numbered or named parts as arguments.
    ("LINE_ABLE_N_ANNUAL_TAXABLE", "ABLE {0} annual taxable"),
    ("LINE_ABLE_N_FOLLOWING_EXPENSES", "ABLE {0} following expenses"),
    ("LINE_ABLE_N_ROLLOVER", "ABLE {0} rollover"),
    ("LINE_ABLE_N_ROLLOVER_EARNINGS", "ABLE {0} rollover earnings"),
    ("LINE_ABLE_CORRECTIVE_EARNINGS", "ABLE corrective earnings"),
    ("LINE_ABLE_SPOUSE_CONTRIBUTION_LIMIT", "ABLE contribution limit, spouse"),
    ("LINE_ABLE_SPOUSE_EXCESS_CONTRIBUTIONS", "ABLE excess contributions, spouse"),
    ("LINE_ABLE_SPOUSE_EXCESS_TAX", "ABLE excess tax, spouse"),
    ("LINE_ABLE_TAXABLE_DISTRIBUTIONS", "ABLE taxable distributions"),
    ("LINE_ABLE_TAXPAYER_CONTRIBUTION_LIMIT", "ABLE contribution limit, taxpayer"),
    ("LINE_ABLE_TAXPAYER_EXCESS_CONTRIBUTIONS", "ABLE excess contributions, taxpayer"),
    ("LINE_ABLE_TAXPAYER_EXCESS_TAX", "ABLE excess tax, taxpayer"),
    ("LINE_ACQUISITION_N_AMOUNT_REALIZED", "Acquisition {0} amount realized"),
    ("LINE_ACQUISITION_N_GAIN", "Acquisition {0} gain"),
    ("LINE_ACQUISITION_N_NONDEDUCTIBLE_LOSS", "Acquisition {0} nondeductible loss"),
    ("LINE_ADDITIONAL_CHILD_CREDIT", "Additional child credit"),
    ("LINE_CHILD_CREDIT_AGI", "Child credit: adjusted gross income"),
    ("LINE_CHILD_CREDIT_FOREIGN_EXCLUSIONS", "Child credit: Form 2555 exclusions added back"),
    ("LINE_CHILD_CREDIT_EXCLUSIONS_TOTAL", "Child credit: excluded income added back"),
    ("LINE_CHILD_CREDIT_MAGI", "Child credit: modified adjusted gross income"),
    ("LINE_CHILD_CREDIT_CHILDREN", "Child credit: qualifying children under 17"),
    ("LINE_CHILD_CREDIT_CHILDREN_AMOUNT", "Child credit: amount for children"),
    ("LINE_CHILD_CREDIT_OTHER_DEPENDENTS", "Child credit: other dependents"),
    ("LINE_CHILD_CREDIT_OTHER_DEPENDENTS_AMOUNT", "Child credit: amount for other dependents"),
    ("LINE_CHILD_CREDIT_BEFORE_PHASEOUT", "Child credit before the phase-out"),
    ("LINE_CHILD_CREDIT_PHASEOUT_THRESHOLD", "Child credit phase-out threshold"),
    ("LINE_CHILD_CREDIT_EXCESS_INCOME", "Child credit: income over the threshold"),
    ("LINE_CHILD_CREDIT_PHASEOUT_REDUCTION", "Child credit phase-out reduction"),
    ("LINE_CHILD_CREDIT_AFTER_PHASEOUT", "Child credit after the phase-out"),
    ("LINE_CHILD_CREDIT_LIMIT", "Child credit tax limit"),
    ("LINE_ADDITIONAL_CHILD_CREDIT_UNUSED", "Additional child credit: unused credit"),
    ("LINE_ADDITIONAL_CHILD_CREDIT_CHILDREN_LIMIT", "Additional child credit: children limit"),
    ("LINE_ADDITIONAL_CHILD_CREDIT_SMALLER", "Additional child credit: smaller of 16a and 16b"),
    ("LINE_ADDITIONAL_CHILD_CREDIT_EARNED_INCOME", "Additional child credit: earned income"),
    (
        "LINE_ADDITIONAL_CHILD_CREDIT_EARNED_OVER_FLOOR",
        "Additional child credit: earned income over $2,500",
    ),
    (
        "LINE_ADDITIONAL_CHILD_CREDIT_EARNED_SHARE",
        "Additional child credit: 15% of earned income over $2,500",
    ),
    (
        "LINE_ADDITIONAL_CHILD_CREDIT_WITHHELD_TAXES",
        "Additional child credit: Social Security and Medicare withheld",
    ),
    ("LINE_ADDITIONAL_CHILD_CREDIT_OTHER_PAYROLL", "Additional child credit: other payroll tax"),
    ("LINE_ADDITIONAL_CHILD_CREDIT_PAYROLL_TOTAL", "Additional child credit: payroll tax total"),
    (
        "LINE_ADDITIONAL_CHILD_CREDIT_EIC_OFFSET",
        "Additional child credit: earned income credit and excess Social Security",
    ),
    (
        "LINE_ADDITIONAL_CHILD_CREDIT_PAYROLL_NET",
        "Additional child credit: payroll tax alternative",
    ),
    ("LINE_ADDITIONAL_CHILD_CREDIT_LARGER", "Additional child credit: larger of 20 and 25"),
    ("LINE_SCHEDULE_EIC_CHILD_N_NAME", "Schedule EIC child {0} name"),
    ("LINE_SCHEDULE_EIC_CHILD_N_SSN", "Schedule EIC child {0} SSN"),
    ("LINE_SCHEDULE_EIC_CHILD_N_DIED", "Schedule EIC child {0} died"),
    ("LINE_SCHEDULE_EIC_CHILD_N_BIRTH_YEAR", "Schedule EIC child {0} year of birth"),
    ("LINE_SCHEDULE_EIC_CHILD_N_STUDENT", "Schedule EIC child {0} student"),
    ("LINE_SCHEDULE_EIC_CHILD_N_DISABLED", "Schedule EIC child {0} disabled"),
    ("LINE_SCHEDULE_EIC_CHILD_N_MONTHS", "Schedule EIC child {0} months"),
    ("LINE_CARE_EXPENSES_ALLOWED", "Care expenses within the limit"),
    ("LINE_CARE_EXPENSES_SMALLEST", "Care expenses limited by earned income"),
    ("LINE_CARE_AGI", "Care credit: adjusted gross income"),
    ("LINE_CARE_PERCENT", "Care credit percentage"),
    ("LINE_CARE_CREDIT_BEFORE_PRIOR", "Care credit before prior-year expenses"),
    ("LINE_CARE_CREDIT_TOTAL", "Care credit including prior-year expenses"),
    ("LINE_CARE_CREDIT_LIMIT", "Care credit tax limit"),
    ("LINE_ADDITIONAL_INCOME", "Additional income"),
    ("LINE_ADDITIONAL_MEDICARE_SELF_EMPLOYMENT", "Additional Medicare self-employment"),
    ("LINE_ADDITIONAL_MEDICARE_TAX", "Additional Medicare tax"),
    ("LINE_ADDITIONAL_MEDICARE_WAGES", "Additional Medicare wages"),
    ("LINE_ADJUSTMENTS", "Adjustments"),
    ("LINE_ADOPTION_N_CREDIT_LIMIT", "Adoption {0} credit limit"),
    ("LINE_ADOPTION_N_EXCLUSION_LIMIT", "Adoption {0} exclusion limit"),
    ("LINE_ADOPTION_BENEFITS_EXCLUDED", "Adoption benefits excluded"),
    ("LINE_ADOPTION_CARRY_N", "Adoption carry {0}"),
    ("LINE_ADOPTION_CARRYFORWARD", "Adoption carryforward"),
    ("LINE_ADOPTION_CHILD_N_BENEFITS_EXCLUDED", "Adoption child {0} benefits excluded"),
    ("LINE_ADOPTION_CHILD_N_BENEFITS_TAXABLE", "Adoption child {0} benefits taxable"),
    (
        "LINE_ADOPTION_CHILD_N_EMPLOYER_EXCLUDED_SPOUSE",
        "Adoption child {0} employer excluded, spouse",
    ),
    (
        "LINE_ADOPTION_CHILD_N_EMPLOYER_EXCLUDED_TAXPAYER",
        "Adoption child {0} employer excluded, taxpayer",
    ),
    ("LINE_ADOPTION_CHILD_N_EMPLOYER_PAYMENT_N", "Adoption child {0} employer payment {1}"),
    (
        "LINE_ADOPTION_CHILD_N_EMPLOYER_TAXABLE_SPOUSE",
        "Adoption child {0} employer taxable, spouse",
    ),
    (
        "LINE_ADOPTION_CHILD_N_EMPLOYER_TAXABLE_TAXPAYER",
        "Adoption child {0} employer taxable, taxpayer",
    ),
    (
        "LINE_ADOPTION_CHILD_N_SPECIAL_NEEDS_DEEMED_EXCESS",
        "Adoption child {0} special needs deemed excess",
    ),
    ("LINE_ADOPTION_EMPLOYER_EXCLUDED_SPOUSE", "Adoption employer excluded, spouse"),
    ("LINE_ADOPTION_EMPLOYER_EXCLUDED_TAXPAYER", "Adoption employer excluded, taxpayer"),
    ("LINE_ADOPTION_EMPLOYER_PAYMENT_N", "Adoption employer payment {0}"),
    ("LINE_ADOPTION_EMPLOYER_TAXABLE_SPOUSE", "Adoption employer taxable, spouse"),
    ("LINE_ADOPTION_EMPLOYER_TAXABLE_TAXPAYER", "Adoption employer taxable, taxpayer"),
    ("LINE_ADOPTION_EXCLUSION_MAGI", "Adoption exclusion MAGI"),
    ("LINE_ADOPTION_EXPIRED_N", "Adoption expired {0}"),
    (
        "LINE_ADOPTION_JOINT_SPECIAL_NEEDS_DEEMED_EXCESS",
        "Adoption joint special needs deemed excess",
    ),
    ("LINE_ADOPTION_NONREFUNDABLE_CREDIT", "Adoption nonrefundable credit"),
    ("LINE_ADOPTION_PRIOR_TAXED_BENEFITS", "Adoption prior taxed benefits"),
    ("LINE_ADOPTION_REFUNDABLE_CREDIT", "Adoption refundable credit"),
    ("LINE_ADOPTION_W2_N_CODE_T_TAXABLE", "Adoption benefits on W-2 {0} taxable (code T)"),
    ("LINE_ADOPTION_WAGE_ADJUSTMENT", "Adoption wage adjustment"),
    ("LINE_AGI", "Adjusted gross income"),
    ("LINE_MOVING_EXCESS_REIMBURSEMENT", "Moving reimbursement above expenses"),
    ("LINE_MOVING_EXPENSE_DEDUCTION", "Moving expense deduction"),
    ("LINE_ALIMONY_PAID_DEDUCTION", "Alimony paid deduction"),
    ("LINE_ALIMONY_RECAPTURE_DEDUCTION", "Alimony recapture deduction"),
    ("LINE_ALIMONY_RECAPTURE_INCOME", "Alimony recapture income"),
    ("LINE_ALTERNATIVE_MINIMUM_TAX", "Alternative minimum tax"),
    ("LINE_AMT_EXEMPTION", "AMT exemption, Form 6251 line 5"),
    ("LINE_AMT_TAXABLE_BASE", "Taxable AMT income, Form 6251 line 6"),
    ("LINE_AMT_TAX_BEFORE_FOREIGN_CREDIT", "AMT before foreign credit, Form 6251 line 7"),
    ("LINE_AMT_FOREIGN_CREDIT", "AMT foreign tax credit, Form 6251 line 8"),
    ("LINE_AMT_TENTATIVE_MINIMUM_TAX", "Tentative minimum tax, Form 6251 line 9"),
    ("LINE_AMT_REGULAR_TAX_OFFSET", "Regular tax comparison, Form 6251 line 10"),
    (
        "EXPLAIN_AMT_COMPARISON",
        "The exemption uses the current filing status and AMT income. The AMT tax after its foreign tax credit is compared with regular tax, before Schedule J averaging and after the applicable credits and repayment taxes.",
    ),
    ("LINE_AMT_CAPITAL_LINE_N", "AMT capital-gain computation line {0}"),
    (
        "EXPLAIN_AMT_CAPITAL_COMPUTATION",
        "The AMT worksheet uses its own gain amounts and the regular-tax capital-gain band occupancy. Rate components keep their cents and the final comparison rounds once. Foreign excluded income is subtracted afterward in its separate worksheet.",
    ),
    ("LINE_AMT_FOREIGN_WORKSHEET_LINE_N", "AMT foreign earned income tax worksheet line {0}"),
    ("LINE_AMT_FOREIGN_WORKSHEET_LINE_2C", "AMT foreign earned income tax worksheet net exclusion"),
    (
        "EXPLAIN_AMT_FOREIGN_COMPUTATION",
        "AMT is figured on taxable income plus the net foreign exclusion, then reduced by the tax on that exclusion. The existing exclusion calculation already removes related deductions and floors the net exclusion at zero.",
    ),
    ("LINE_ALTERNATIVE_MINIMUM_TAXABLE_INCOME", "Alternative minimum taxable income"),
    ("LINE_AMT_BUSINESS_DEPRECIATION_ADJUSTMENT", "AMT business depreciation adjustment"),
    ("LINE_AMT_FARM_SUPPLIES_ADJUSTMENT", "AMT farm prepaid-supply adjustment"),
    ("LINE_FARM_N_SUPPLIES_AMT_DEDUCTION", "Farm {0} AMT supply deduction"),
    ("LINE_FARM_N_SUPPLIES_AMT_REMAINING", "Farm {0} undeducted AMT supply cost"),
    (
        "EXPLAIN_AMT_FARM_SUPPLIES_ADJUSTMENT",
        "Regular supply deductions less the separate AMT allowance, from the same retained supply and farming-expense records.",
    ),
    (
        "EXPLAIN_FARM_SUPPLIES_AMT_DEDUCTION",
        "Current allowed supply costs using the AMT other-expense ceiling and separate retained AMT cost.",
    ),
    (
        "EXPLAIN_FARM_SUPPLIES_AMT_REMAINING",
        "Net unrecovered supply cost retained for AMT after its current allowance.",
    ),
    ("LINE_AMT_FARM_CONSERVATION_ADJUSTMENT", "AMT farm conservation adjustment"),
    ("LINE_AMT_CAPITAL_ADJUSTMENT", "AMT capital adjustment"),
    ("LINE_AMT_CAPITAL_GAIN", "AMT capital gain"),
    ("LINE_AMT_CAPITAL_LONG_CARRYFORWARD", "AMT capital long carryforward"),
    ("LINE_AMT_CAPITAL_LOSS_CARRYFORWARD", "AMT capital loss carryforward"),
    ("LINE_AMT_CAPITAL_SHORT_CARRYFORWARD", "AMT capital short carryforward"),
    ("LINE_AMT_CURRENT_NET_OPERATING_LOSS", "AMT current net operating loss"),
    ("LINE_AMT_CURRENT_NOL_DEBT_REDUCTION", "AMT current NOL debt reduction"),
    (
        "LINE_AMT_ESTATE_TRUST_N_PASSIVE_ACTIVITY_INCOME",
        "AMT estate trust {0} passive activity income",
    ),
    ("LINE_AMT_EXCESS_BUSINESS_LOSS_CARRYFORWARD", "AMT excess business loss carryforward"),
    ("LINE_AMT_INVESTMENT_ELECTED_CAPITAL_GAIN", "AMT investment elected capital gain"),
    (
        "LINE_AMT_INVESTMENT_ELECTED_QUALIFIED_DIVIDENDS",
        "AMT investment elected qualified dividends",
    ),
    ("LINE_AMT_INVESTMENT_INTEREST_CARRYFORWARD", "AMT investment interest carryforward"),
    ("LINE_AMT_INVESTMENT_INTEREST_DEDUCTION", "AMT investment interest deduction"),
    ("LINE_AMT_ISO_COMPENSATION_ADJUSTMENT", "AMT ISO compensation adjustment"),
    ("LINE_AMT_NOL_CARRYFORWARD", "AMT NOL carryforward"),
    ("LINE_AMT_PARTNERSHIP_N_ACTIVITY_NET", "AMT partnership {0} activity net"),
    ("LINE_AMT_PARTNERSHIP_N_PASSIVE_SUSPENDED", "AMT partnership {0} passive suspended"),
    ("LINE_AMT_PERSONAL_RENTAL_N_ALLOWED_EXPENSES", "AMT personal rental {0} allowed expenses"),
    (
        "LINE_AMT_PERSONAL_RENTAL_N_PRIOR_SUSPENDED_EXPENSES",
        "AMT personal rental {0} prior suspended expenses",
    ),
    ("LINE_AMT_PERSONAL_RENTAL_N_SUSPENDED_LOSS", "AMT personal rental {0} suspended loss"),
    ("LINE_AMT_PERSONAL_RENTAL_PASSIVE_INCOME", "AMT personal rental passive income"),
    ("LINE_AMT_PERSONAL_SALE_DISALLOWED_LOSS", "AMT personal sale disallowed loss"),
    ("LINE_AMT_RATE_N_GAIN", "AMT rate {0} gain"),
    ("LINE_AMT_RENTAL_N_DEPRECIATION", "AMT rental {0} depreciation"),
    (
        "LINE_AMT_RENTAL_N_HOME_DEPRECIATION_CARRYOVER",
        "AMT rental {0} home casualty and depreciation carryover",
    ),
    ("LINE_AMT_RENTAL_N_HOME_OPERATING_CARRYOVER", "AMT rental {0} home operating carryover"),
    ("LINE_AMT_RENTAL_N_SUSPENDED_LOSS", "AMT rental {0} suspended loss"),
    ("LINE_AMT_RENTAL_ADJUSTMENT", "AMT rental adjustment"),
    ("LINE_AMT_RENTAL_INCOME", "AMT rental income"),
    ("LINE_AMT_RENTAL_MAGI", "AMT rental MAGI"),
    ("LINE_AMT_RENTAL_SUSPENDED_LOSS", "AMT rental suspended loss"),
    ("LINE_AMT_SECTION1231_N_NEXT_LOOKBACK", "AMT section 1231 {0} next lookback"),
    ("LINE_AMT_SECTION1231_CURRENT_LOSS_HISTORY", "AMT section 1231 current loss history"),
    ("LINE_AMT_TOTAL_NOL_CARRYFORWARD", "AMT total NOL carryforward"),
    ("LINE_AMT_UNRECAPTURED_SECTION_N_GAIN", "AMT unrecaptured section {0} gain"),
    ("LINE_ANNUITY_N_BASIS_INCREASE", "Annuity {0} basis increase"),
    ("LINE_ANNUITY_N_BASIS_RECOVERY", "Annuity {0} basis recovery"),
    ("LINE_ANNUITY_N_EARLIER_RECOVERY", "Annuity {0} earlier recovery"),
    ("LINE_ANNUITY_N_EXPECTED_RETURN", "Annuity {0} expected return"),
    ("LINE_ANNUITY_N_REFUND_ADJUSTMENT", "Annuity {0} refund adjustment"),
    ("LINE_ANNUITY_N_REMAINING_BASIS", "Annuity {0} remaining basis"),
    ("LINE_ANNUITY_N_ROLLED_BASIS", "Annuity {0} rolled basis"),
    ("LINE_ANNUITY_N_SURRENDER_LOSS", "Annuity {0} surrender loss"),
    ("LINE_ANNUITY_N_TAXABLE_WITHDRAWAL", "Annuity {0} taxable withdrawal"),
    ("LINE_ANNUITY_LOSS_N", "Annuity loss {0}"),
    ("LINE_ARCHER_N_N_EXCESS_ACCOUNT_VALUE", "Archer {0} {1} excess account value"),
    ("LINE_ARCHER_N_N_UNCORRECTED_EMPLOYER_EXCESS", "Archer {0} {1} uncorrected employer excess"),
    (
        "LINE_ARCHER_N_N_UNCORRECTED_EMPLOYER_EXCESS_OTHER_INCOME",
        "Archer {0} {1} uncorrected employer excess other income",
    ),
    ("LINE_ARCHER_N_N_UNCORRECTED_PERSONAL_EXCESS", "Archer {0} {1} uncorrected personal excess"),
    ("LINE_ARCHER_N_CORRECTED_CONTRIBUTIONS", "Archer {0} corrected contributions"),
    ("LINE_ARCHER_N_EXCESS_ACCOUNT_VALUE", "Archer {0} excess account value"),
    ("LINE_ARCHER_N_EXCESS_CONTRIBUTION_TAX", "Archer {0} excess contribution tax"),
    (
        "LINE_ARCHER_N_PRIOR_DESIGNATED_CORRECTED_CONTRIBUTIONS",
        "Archer {0} prior designated corrected contributions",
    ),
    (
        "LINE_ARCHER_N_PRIOR_DESIGNATED_EMPLOYER_EXCESS",
        "Archer {0} prior designated employer excess",
    ),
    ("LINE_ARCHER_N_PRIOR_EXCESS_DEDUCTED_N", "Archer {0} prior excess deducted {1}"),
    ("LINE_ARCHER_N_PRIOR_EXCESS_REMAINING_N", "Archer {0} prior excess remaining {1}"),
    (
        "LINE_ARCHER_N_PRIOR_EXCESS_USED_BY_DISTRIBUTIONS",
        "Archer {0} prior excess used by distributions",
    ),
    ("LINE_ARCHER_N_UNCORRECTED_EMPLOYER_EXCESS", "Archer {0} uncorrected employer excess"),
    (
        "LINE_ARCHER_N_UNCORRECTED_EMPLOYER_EXCESS_OTHER_INCOME",
        "Archer {0} uncorrected employer excess other income",
    ),
    ("LINE_ARCHER_N_UNCORRECTED_PERSONAL_EXCESS", "Archer {0} uncorrected personal excess"),
    ("LINE_ARCHER_CONTRIBUTION_N_COMPENSATION", "Archer contribution {0} compensation"),
    ("LINE_ARCHER_CONTRIBUTION_N_COVERAGE_LIMIT", "Archer contribution {0} coverage limit"),
    ("LINE_ARCHER_CONTRIBUTION_N_DEDUCTION", "Archer contribution {0} deduction"),
    ("LINE_ARCHER_CORRECTION_EARNINGS", "Archer correction earnings"),
    (
        "LINE_ARCHER_CORRECTION_EMPLOYER_PRINCIPAL_OTHER_INCOME",
        "Archer correction employer principal other income",
    ),
    ("LINE_ARCHER_MSA_N_DEEMED", "Archer MSA {0} deemed"),
    ("LINE_ARCHER_MSA_N_GROSS", "Archer MSA {0} gross"),
    ("LINE_ARCHER_MSA_N_MEDICAL", "Archer MSA {0} medical"),
    ("LINE_ARCHER_MSA_N_PENALTY_BASE", "Archer MSA {0} penalty base"),
    ("LINE_ARCHER_MSA_N_ROLLOVER", "Archer MSA {0} rollover"),
    ("LINE_ARCHER_MSA_N_TAXABLE", "Archer MSA {0} taxable"),
    ("LINE_ARCHER_MSA_ADDITIONAL_TAX", "Archer MSA additional tax"),
    ("LINE_ARCHER_MSA_DEDUCTION", "Archer MSA deduction"),
    ("LINE_ARCHER_MSA_TAXABLE_DISTRIBUTIONS", "Archer MSA taxable distributions"),
    (
        "LINE_ARCHER_PRIOR_DESIGNATED_EMPLOYER_EXCESS_OTHER_INCOME",
        "Archer prior designated employer excess other income",
    ),
    ("LINE_ARCHER_PRIOR_EXCESS_DEDUCTED_N", "Archer prior excess deducted {0}"),
    ("LINE_ARCHER_PRIOR_EXCESS_REMAINING_N", "Archer prior excess remaining {0}"),
    ("LINE_ARCHER_PRIOR_EXCESS_USED_BY_DISTRIBUTIONS", "Archer prior excess used by distributions"),
    ("LINE_ATNOL_N_CARRYFORWARD", "ATNOL {0} carryforward (alternative tax net operating loss)"),
    (
        "LINE_ATNOL_N_DEBT_REDUCTION",
        "ATNOL {0} debt reduction (alternative tax net operating loss)",
    ),
    ("LINE_ATNOL_N_EXPIRED", "ATNOL {0} expired (alternative tax net operating loss)"),
    ("LINE_ATNOL_N_OPENING", "ATNOL {0} opening (alternative tax net operating loss)"),
    ("LINE_ATNOL_N_USED", "ATNOL {0} used (alternative tax net operating loss)"),
    ("LINE_BAD_DEBT_N_SHORT_TERM_LOSS", "Bad debt {0} short-term loss"),
    ("LINE_BIRTH_ADOPTION_USED_N_N", "Birth adoption used {0} {1}"),
    ("LINE_BOND_CREDIT_DEEMED_INTEREST", "Bond credit deemed interest"),
    ("LINE_BUSINESS_N_ASSET_N_AMT_DEPRECIATION", "Business {0} asset {1} AMT depreciation"),
    ("LINE_BUSINESS_N_ASSET_N_AMT_REMAINING_BASIS", "Business {0} asset {1} AMT remaining basis"),
    ("LINE_BUSINESS_N_ASSET_N_BONUS", "Business {0} asset {1} bonus"),
    (
        "LINE_BUSINESS_N_ASSET_N_DISPOSAL_N_RECAPTURE",
        "Business {0} asset {1} disposal {2} recapture",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_DISPOSAL_ADJUSTED_BASIS",
        "Business {0} asset {1} disposal adjusted basis",
    ),
    ("LINE_BUSINESS_N_ASSET_N_DISPOSAL_COST", "Business {0} asset {1} disposal cost"),
    (
        "LINE_BUSINESS_N_ASSET_N_DISPOSAL_DEPRECIATION",
        "Business {0} asset {1} disposal depreciation",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_DISPOSAL_FORM6252_ADJUSTED_BASIS",
        "Business {0} asset {1} disposal Form 6252 adjusted basis",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_DISPOSAL_FORM6252_ASSUMED_DEBT",
        "Business {0} asset {1} disposal Form 6252 assumed debt",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_DISPOSAL_FORM6252_CONTRACT_PRICE",
        "Business {0} asset {1} disposal Form 6252 contract price",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_DISPOSAL_FORM6252_COST",
        "Business {0} asset {1} disposal Form 6252 cost",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_DISPOSAL_FORM6252_DEEMED_PAYMENT",
        "Business {0} asset {1} disposal Form 6252 deemed payment",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_DISPOSAL_FORM6252_DEPRECIATION",
        "Business {0} asset {1} disposal Form 6252 depreciation",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_DISPOSAL_FORM6252_EXCESS_DEBT",
        "Business {0} asset {1} disposal Form 6252 excess debt",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_DISPOSAL_FORM6252_EXPENSES",
        "Business {0} asset {1} disposal Form 6252 expenses",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_DISPOSAL_FORM6252_GAIN",
        "Business {0} asset {1} disposal Form 6252 gain",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_DISPOSAL_FORM6252_GROSS_PROFIT",
        "Business {0} asset {1} disposal Form 6252 gross profit",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_DISPOSAL_FORM6252_PROFIT",
        "Business {0} asset {1} disposal Form 6252 profit before exclusion",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_DISPOSAL_FORM6252_PERCENTAGE",
        "Business {0} asset {1} disposal Form 6252 gross profit percentage",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_DISPOSAL_FORM6252_INCOME",
        "Business {0} asset {1} disposal Form 6252 income",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_DISPOSAL_FORM6252_NET_PRICE",
        "Business {0} asset {1} disposal Form 6252 net price",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_DISPOSAL_FORM6252_ORDINARY",
        "Business {0} asset {1} disposal Form 6252 ordinary",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_DISPOSAL_FORM6252_PAYMENTS",
        "Business {0} asset {1} disposal Form 6252 payments",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_DISPOSAL_FORM6252_PRICE",
        "Business {0} asset {1} disposal Form 6252 price",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_DISPOSAL_FORM6252_RECAPTURE",
        "Business {0} asset {1} disposal Form 6252 recapture",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_DISPOSAL_FORM6252_TOTAL",
        "Business {0} asset {1} disposal Form 6252 total",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_DISPOSAL_FORM6252_TOTAL_PAYMENTS",
        "Business {0} asset {1} disposal Form 6252 total payments",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_DISPOSAL_FORM6252_UNRECAPTURED_N",
        "Business {0} asset {1} disposal Form 6252 unrecaptured {2}",
    ),
    ("LINE_BUSINESS_N_ASSET_N_DISPOSAL_GAIN", "Business {0} asset {1} disposal gain"),
    (
        "LINE_BUSINESS_N_ASSET_N_DISPOSAL_INSTALLMENT_GAIN",
        "Business {0} asset {1} disposal installment gain",
    ),
    ("LINE_BUSINESS_N_ASSET_N_DISPOSAL_ORDINARY", "Business {0} asset {1} disposal ordinary"),
    ("LINE_BUSINESS_N_ASSET_N_DISPOSAL_SALE_PRICE", "Business {0} asset {1} disposal sale price"),
    (
        "LINE_BUSINESS_N_ASSET_N_DISPOSAL_SECTION1231",
        "Business {0} asset {1} disposal section 1231",
    ),
    ("LINE_BUSINESS_N_ASSET_N_LISTED_BUSINESS_USE", "Business {0} asset {1} listed business use"),
    ("LINE_BUSINESS_N_ASSET_N_LISTED_COST", "Business {0} asset {1} listed cost"),
    ("LINE_BUSINESS_N_ASSET_N_LISTED_DEPRECIATION", "Business {0} asset {1} listed depreciation"),
    (
        "LINE_BUSINESS_N_ASSET_N_LISTED_DEPRECIATION_BASIS",
        "Business {0} asset {1} listed depreciation basis",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_LISTED_RECAPTURE",
        "Business {0} asset {1} recapture of excess depreciation",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_LISTED_RECOMPUTED",
        "Business {0} asset {1} straight-line depreciation for the earlier years",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_LISTED_PRIOR_DEPRECIATION",
        "Business {0} asset {1} depreciation allowed in earlier years",
    ),
    ("LINE_BUSINESS_N_ASSET_N_LISTED_SECTION179", "Business {0} asset {1} listed section 179"),
    (
        "LINE_BUSINESS_N_ASSET_N_LISTED_SPECIAL_ALLOWANCE",
        "Business {0} asset {1} listed special allowance",
    ),
    ("LINE_BUSINESS_N_ASSET_N_LISTED_TYPE", "Business {0} asset {1} listed type"),
    ("LINE_BUSINESS_N_ASSET_N_MACRS", "Business {0} asset {1} MACRS"),
    ("LINE_BUSINESS_N_ASSET_N_REMAINING_BASIS", "Business {0} asset {1} remaining basis"),
    ("LINE_BUSINESS_N_ASSET_N_SECTION179", "Business {0} asset {1} section 179"),
    (
        "LINE_BUSINESS_N_ASSET_N_SECTION179_CARRYFORWARD",
        "Business {0} asset {1} section 179 carryforward",
    ),
    ("LINE_BUSINESS_N_CAR_AND_TRUCK", "Business {0} car and truck"),
    (
        "LINE_BUSINESS_N_CARD_ISSUER_RETAINED_SCHEDULE_C_N",
        "Business {0} card issuer retained Schedule C {1}",
    ),
    ("LINE_BUSINESS_N_CASH_FUEL_TAX_RECOVERY", "Business {0} cash fuel tax recovery"),
    (
        "LINE_BUSINESS_N_CASH_FUEL_TAX_RECOVERY_EXCLUDED",
        "Business {0} cash fuel tax recovery excluded",
    ),
    ("LINE_BUSINESS_N_CASH_FUEL_TAX_RECOVERY_GROSS", "Business {0} cash fuel tax recovery gross"),
    ("LINE_BUSINESS_N_COST_OF_GOODS_SOLD", "Business {0} cost of goods sold"),
    ("LINE_BUSINESS_N_DEPRECIATION", "Business {0} depreciation"),
    ("LINE_BUSINESS_N_DEPRECIATION_TOTAL", "Business {0} depreciation total"),
    ("LINE_BUSINESS_N_EXCLUDED_CHILD_PREMIUMS", "Business {0} excluded child premiums"),
    ("LINE_BUSINESS_N_EXPENSE_N_DE_MINIMIS", "Business {0} expense {1} de minimis"),
    (
        "LINE_BUSINESS_N_EXPENSE_LINE10_COMMISSIONS_AND_FEES",
        "Business {0} expense line 10 commissions and fees",
    ),
    (
        "LINE_BUSINESS_N_EXPENSE_LINE11_CONTRACT_LABOR",
        "Business {0} expense line 11 contract labor",
    ),
    (
        "LINE_BUSINESS_N_EXPENSE_LINE14_EMPLOYEE_BENEFIT_PROGRAMS",
        "Business {0} expense line 14 employee benefit programs",
    ),
    ("LINE_BUSINESS_N_EXPENSE_LINE15_INSURANCE", "Business {0} expense line 15 insurance"),
    (
        "LINE_BUSINESS_N_EXPENSE_LINE16A_MORTGAGE_INTEREST",
        "Business {0} expense line 16a mortgage interest",
    ),
    (
        "LINE_BUSINESS_N_EXPENSE_LINE16C_OTHER_INTEREST",
        "Business {0} expense line 16c other interest",
    ),
    (
        "LINE_BUSINESS_N_EXPENSE_LINE17_LEGAL_AND_PROFESSIONAL",
        "Business {0} expense line 17 legal and professional",
    ),
    ("LINE_BUSINESS_N_EXPENSE_LINE18_OFFICE", "Business {0} expense line 18 office"),
    (
        "LINE_BUSINESS_N_EXPENSE_LINE20A_RENT_VEHICLES_EQUIPMENT",
        "Business {0} expense line 20a rent vehicles equipment",
    ),
    ("LINE_BUSINESS_N_EXPENSE_LINE20B_RENT_OTHER", "Business {0} expense line 20b rent other"),
    ("LINE_BUSINESS_N_EXPENSE_LINE21_REPAIRS", "Business {0} expense line 21 repairs"),
    ("LINE_BUSINESS_N_EXPENSE_LINE22_SUPPLIES", "Business {0} expense line 22 supplies"),
    (
        "LINE_BUSINESS_N_EXPENSE_LINE23_TAXES_AND_LICENSES",
        "Business {0} expense line 23 taxes and licenses",
    ),
    ("LINE_BUSINESS_N_EXPENSE_LINE24A_TRAVEL", "Business {0} expense line 24a travel"),
    ("LINE_BUSINESS_N_EXPENSE_LINE24B_MEALS", "Business {0} expense line 24b meals"),
    ("LINE_BUSINESS_N_EXPENSE_LINE25_UTILITIES", "Business {0} expense line 25 utilities"),
    ("LINE_BUSINESS_N_EXPENSE_LINE26_WAGES", "Business {0} expense line 26 wages"),
    ("LINE_BUSINESS_N_EXPENSE_LINE27B_OTHER", "Business {0} expense line 27b other"),
    ("LINE_BUSINESS_N_EXPENSE_LINE8_ADVERTISING", "Business {0} expense line 8 advertising"),
    ("LINE_BUSINESS_N_FOREIGN_NET_INCOME", "Business {0} foreign net income"),
    ("LINE_BUSINESS_N_FORM5884_LINE1A", "Business {0} Form 5884 line 1a"),
    ("LINE_BUSINESS_N_FORM5884_LINE1B", "Business {0} Form 5884 line 1b"),
    ("LINE_BUSINESS_N_FORM5884_LINE1C", "Business {0} Form 5884 line 1c"),
    ("LINE_BUSINESS_N_FORM5884_LINE2", "Business {0} Form 5884 line 2"),
    (
        "LINE_BUSINESS_N_FORM8846_ACTUAL_EMPLOYER_TIP_FICA_PAID",
        "Business {0} Form 8846 actual employer tip FICA paid",
    ),
    ("LINE_BUSINESS_N_FORM8846_HOURLY_BASIS", "Business {0} Form 8846 hourly basis"),
    ("LINE_BUSINESS_N_FORM8846_LINE1", "Business {0} Form 8846 line 1"),
    ("LINE_BUSINESS_N_FORM8846_LINE2", "Business {0} Form 8846 line 2"),
    ("LINE_BUSINESS_N_FORM8846_LINE3", "Business {0} Form 8846 line 3"),
    ("LINE_BUSINESS_N_FORM8846_LINE4", "Business {0} Form 8846 line 4"),
    (
        "LINE_BUSINESS_N_FORM8846_REMAINING_EMPLOYER_FICA_DEDUCTION",
        "Business {0} Form 8846 remaining employer FICA deduction",
    ),
    ("LINE_BUSINESS_N_FORM8881_LINE1", "Business {0} Form 8881 line 1"),
    ("LINE_BUSINESS_N_FORM8881_LINE2", "Business {0} Form 8881 line 2"),
    ("LINE_BUSINESS_N_FORM8881_LINE3", "Business {0} Form 8881 line 3"),
    ("LINE_BUSINESS_N_FORM8881_LINE4", "Business {0} Form 8881 line 4"),
    ("LINE_BUSINESS_N_FORM8881_LINE5", "Business {0} Form 8881 line 5"),
    ("LINE_BUSINESS_N_FORM8881_LINE9", "Business {0} Form 8881 line 9"),
    ("LINE_BUSINESS_N_FORM8941_LINE1", "Business {0} Form 8941 line 1"),
    ("LINE_BUSINESS_N_FORM8941_LINE2", "Business {0} Form 8941 line 2"),
    ("LINE_BUSINESS_N_FORM8941_LINE3", "Business {0} Form 8941 line 3"),
    ("LINE_BUSINESS_N_FORM8941_LINE4", "Business {0} Form 8941 line 4"),
    ("LINE_BUSINESS_N_FORM8941_LINE5", "Business {0} Form 8941 line 5"),
    ("LINE_BUSINESS_N_FORM8941_LINE6", "Business {0} Form 8941 line 6"),
    ("LINE_BUSINESS_N_FORM8941_LINE7", "Business {0} Form 8941 line 7"),
    ("LINE_BUSINESS_N_FORM8941_LINE8", "Business {0} Form 8941 line 8"),
    ("LINE_BUSINESS_N_FORM8941_LINE9", "Business {0} Form 8941 line 9"),
    ("LINE_BUSINESS_N_FORM8941_LINE10", "Business {0} Form 8941 line 10"),
    ("LINE_BUSINESS_N_FORM8941_LINE11", "Business {0} Form 8941 line 11"),
    ("LINE_BUSINESS_N_FORM8941_LINE12", "Business {0} Form 8941 line 12"),
    ("LINE_BUSINESS_N_FORM8941_LINE13", "Business {0} Form 8941 line 13"),
    ("LINE_BUSINESS_N_FORM8941_LINE14", "Business {0} Form 8941 line 14"),
    ("LINE_BUSINESS_N_FORM8941_LINE15", "Business {0} Form 8941 line 15"),
    ("LINE_BUSINESS_N_FORM8941_LINE16", "Business {0} Form 8941 line 16"),
    ("LINE_BUSINESS_N_FORM8994_LINE1A", "Business {0} Form 8994 line 1a"),
    ("LINE_BUSINESS_N_FORM8994_LINE1B", "Business {0} Form 8994 line 1b"),
    ("LINE_BUSINESS_N_FORM8994_LINE2", "Business {0} Form 8994 line 2"),
    ("LINE_BUSINESS_N_FORM8994_LINE3", "Business {0} Form 8994 line 3"),
    ("LINE_BUSINESS_N_FORM_N_ALLOWED", "Business {0} Form {1} allowed"),
    ("LINE_BUSINESS_N_FORM_N_BASIS", "Business {0} Form {1} basis"),
    ("LINE_BUSINESS_N_FORM_N_BUILDING", "Business {0} Form {1} building"),
    ("LINE_BUSINESS_N_FORM_N_BUSINESS_BASIS", "Business {0} Form {1} business basis"),
    ("LINE_BUSINESS_N_FORM_N_DEPRECIATION", "Business {0} Form {1} depreciation"),
    (
        "LINE_BUSINESS_N_FORM_N_DEPRECIATION_ALLOWABLE",
        "Business {0} Form {1} depreciation allowable",
    ),
    ("LINE_BUSINESS_N_FORM_N_DEPRECIATION_ALLOWED", "Business {0} Form {1} depreciation allowed"),
    (
        "LINE_BUSINESS_N_FORM_N_DEPRECIATION_CARRYOVER",
        "Business {0} Form {1} depreciation carryover",
    ),
    ("LINE_BUSINESS_N_FORM_N_DEPRECIATION_LIMIT", "Business {0} Form {1} depreciation limit"),
    ("LINE_BUSINESS_N_FORM_N_DEPRECIATION_TOTAL", "Business {0} Form {1} depreciation total"),
    ("LINE_BUSINESS_N_FORM_N_DIRECT_REPAIRS", "Business {0} Form {1} direct repairs"),
    ("LINE_BUSINESS_N_FORM_N_DIRECT_TOTAL", "Business {0} Form {1} direct total"),
    (
        "LINE_BUSINESS_N_FORM_N_DEDUCTIBLE_INDIRECT",
        "Business {0} Form {1} deductible indirect expenses",
    ),
    (
        "LINE_BUSINESS_N_FORM_N_DEDUCTIBLE_DIRECT",
        "Business {0} Form {1} deductible direct expenses",
    ),
    ("LINE_BUSINESS_N_FORM_N_EXCESS_INTEREST", "Business {0} Form {1} excess interest"),
    ("LINE_BUSINESS_N_FORM_N_EXCESS_TAXES", "Business {0} Form {1} excess taxes"),
    ("LINE_BUSINESS_N_FORM_N_INCOME", "Business {0} Form {1} income"),
    ("LINE_BUSINESS_N_FORM_N_INDIRECT_SHARE", "Business {0} Form {1} indirect share"),
    ("LINE_BUSINESS_N_FORM_N_INDIRECT_TOTAL", "Business {0} Form {1} indirect total"),
    ("LINE_BUSINESS_N_FORM_N_INSURANCE", "Business {0} Form {1} insurance"),
    ("LINE_BUSINESS_N_FORM_N_LAND", "Business {0} Form {1} land"),
    ("LINE_BUSINESS_N_FORM_N_LIMIT", "Business {0} Form {1} limit"),
    ("LINE_BUSINESS_N_FORM_N_MORTGAGE_INTEREST", "Business {0} Form {1} mortgage interest"),
    ("LINE_BUSINESS_N_FORM_N_OPERATING_ALLOWED", "Business {0} Form {1} operating allowed"),
    ("LINE_BUSINESS_N_FORM_N_OPERATING_CARRYOVER", "Business {0} Form {1} operating carryover"),
    ("LINE_BUSINESS_N_FORM_N_OPERATING_SHARE", "Business {0} Form {1} operating share"),
    ("LINE_BUSINESS_N_FORM_N_OPERATING_TOTAL", "Business {0} Form {1} operating total"),
    ("LINE_BUSINESS_N_FORM_N_OTHER", "Business {0} Form {1} other"),
    ("LINE_BUSINESS_N_FORM_N_PRIOR_DEPRECIATION", "Business {0} Form {1} prior depreciation"),
    ("LINE_BUSINESS_N_FORM_N_PRIOR_OPERATING", "Business {0} Form {1} prior operating"),
    ("LINE_BUSINESS_N_FORM_N_REAL_ESTATE_TAXES", "Business {0} Form {1} real estate taxes"),
    ("LINE_BUSINESS_N_FORM_N_RENT", "Business {0} Form {1} rent"),
    ("LINE_BUSINESS_N_FORM_N_REPAIRS", "Business {0} Form {1} repairs"),
    ("LINE_BUSINESS_N_FORM_N_UNLIMITED", "Business {0} Form {1} unlimited"),
    ("LINE_BUSINESS_N_FORM_N_UTILITIES", "Business {0} Form {1} utilities"),
    (
        "LINE_BUSINESS_N_FORM_N_WORKSHEET_BUSINESS_TAXES",
        "Business {0} Form {1} worksheet business taxes",
    ),
    ("LINE_BUSINESS_N_FUEL_11A", "Business {0} fuel 11a"),
    ("LINE_BUSINESS_N_FUEL_11B", "Business {0} fuel 11b"),
    ("LINE_BUSINESS_N_FUEL_11C", "Business {0} fuel 11c"),
    ("LINE_BUSINESS_N_FUEL_11D", "Business {0} fuel 11d"),
    ("LINE_BUSINESS_N_FUEL_11E", "Business {0} fuel 11e"),
    ("LINE_BUSINESS_N_FUEL_11F", "Business {0} fuel 11f"),
    ("LINE_BUSINESS_N_FUEL_11G", "Business {0} fuel 11g"),
    ("LINE_BUSINESS_N_FUEL_11H", "Business {0} fuel 11h"),
    ("LINE_BUSINESS_N_FUEL_13A", "Business {0} fuel 13a"),
    ("LINE_BUSINESS_N_FUEL_13B", "Business {0} fuel 13b"),
    ("LINE_BUSINESS_N_FUEL_13C_N", "Business {0} fuel 13c {1}"),
    ("LINE_BUSINESS_N_FUEL_14A", "Business {0} fuel 14a"),
    ("LINE_BUSINESS_N_FUEL_14B", "Business {0} fuel 14b"),
    ("LINE_BUSINESS_N_FUEL_15A", "Business {0} fuel 15a"),
    ("LINE_BUSINESS_N_FUEL_16A", "Business {0} fuel 16a"),
    ("LINE_BUSINESS_N_FUEL_16B", "Business {0} fuel 16b"),
    ("LINE_BUSINESS_N_FUEL_1A", "Business {0} fuel 1a"),
    ("LINE_BUSINESS_N_FUEL_1B", "Business {0} fuel 1b"),
    ("LINE_BUSINESS_N_FUEL_1C", "Business {0} fuel 1c"),
    ("LINE_BUSINESS_N_FUEL_1D", "Business {0} fuel 1d"),
    ("LINE_BUSINESS_N_FUEL_2A", "Business {0} fuel 2a"),
    ("LINE_BUSINESS_N_FUEL_2B", "Business {0} fuel 2b"),
    ("LINE_BUSINESS_N_FUEL_2C", "Business {0} fuel 2c"),
    ("LINE_BUSINESS_N_FUEL_2D", "Business {0} fuel 2d"),
    ("LINE_BUSINESS_N_FUEL_3A", "Business {0} fuel 3a"),
    ("LINE_BUSINESS_N_FUEL_3B", "Business {0} fuel 3b"),
    ("LINE_BUSINESS_N_FUEL_3C", "Business {0} fuel 3c"),
    ("LINE_BUSINESS_N_FUEL_3D", "Business {0} fuel 3d"),
    ("LINE_BUSINESS_N_FUEL_3E", "Business {0} fuel 3e"),
    ("LINE_BUSINESS_N_FUEL_4A", "Business {0} fuel 4a"),
    ("LINE_BUSINESS_N_FUEL_4B", "Business {0} fuel 4b"),
    ("LINE_BUSINESS_N_FUEL_4C", "Business {0} fuel 4c"),
    ("LINE_BUSINESS_N_FUEL_4D", "Business {0} fuel 4d"),
    ("LINE_BUSINESS_N_FUEL_4E", "Business {0} fuel 4e"),
    ("LINE_BUSINESS_N_FUEL_4F", "Business {0} fuel 4f"),
    ("LINE_BUSINESS_N_FUEL_5A", "Business {0} fuel 5a"),
    ("LINE_BUSINESS_N_FUEL_5B", "Business {0} fuel 5b"),
    ("LINE_BUSINESS_N_FUEL_5C", "Business {0} fuel 5c"),
    ("LINE_BUSINESS_N_FUEL_5D", "Business {0} fuel 5d"),
    ("LINE_BUSINESS_N_FUEL_5E", "Business {0} fuel 5e"),
    ("LINE_BUSINESS_N_FUEL_6A", "Business {0} fuel 6a"),
    ("LINE_BUSINESS_N_FUEL_6B", "Business {0} fuel 6b"),
    ("LINE_BUSINESS_N_FUEL_7A", "Business {0} fuel 7a"),
    ("LINE_BUSINESS_N_FUEL_7B", "Business {0} fuel 7b"),
    ("LINE_BUSINESS_N_FUEL_7C", "Business {0} fuel 7c"),
    ("LINE_BUSINESS_N_FUEL_8A", "Business {0} fuel 8a"),
    ("LINE_BUSINESS_N_FUEL_8B", "Business {0} fuel 8b"),
    ("LINE_BUSINESS_N_FUEL_8C", "Business {0} fuel 8c"),
    ("LINE_BUSINESS_N_FUEL_8D", "Business {0} fuel 8d"),
    ("LINE_BUSINESS_N_FUEL_8E", "Business {0} fuel 8e"),
    ("LINE_BUSINESS_N_FUEL_8F", "Business {0} fuel 8f"),
    ("LINE_BUSINESS_N_RETURNS", "Business {0} returns and allowances"),
    ("LINE_BUSINESS_N_NET_RECEIPTS", "Business {0} net receipts"),
    ("LINE_BUSINESS_N_GROSS_PROFIT", "Business {0} gross profit"),
    ("LINE_BUSINESS_N_DE_MINIMIS", "Business {0} de minimis purchases"),
    ("LINE_BUSINESS_N_DEPLETION", "Business {0} depletion"),
    ("LINE_BUSINESS_N_DEPLETION_CARRYOVER", "Business {0} depletion carried to 2027"),
    ("LINE_AMT_BUSINESS_N_DEPLETION_CARRYOVER", "Business {0} AMT depletion carried to 2027"),
    ("LINE_BUSINESS_N_OWNER_PLAN_DEDUCTION", "Business {0} owner retirement plan deduction"),
    ("LINE_PARTNERSHIP_N_OWNER_PLAN_DEDUCTION", "Partnership {0} retirement plan deduction"),
    ("LINE_PARTNER_OWNER_PLAN_N_STEP1", "Partnership {0} retirement worksheet step 1"),
    ("LINE_PARTNER_OWNER_PLAN_N_STEP2", "Partnership {0} retirement worksheet step 2"),
    ("LINE_PARTNER_OWNER_PLAN_N_STEP3", "Partnership {0} retirement worksheet step 3"),
    ("LINE_PARTNER_OWNER_PLAN_N_STEP5", "Partnership {0} retirement worksheet step 5"),
    ("LINE_PARTNER_OWNER_PLAN_N_STEP6", "Partnership {0} retirement worksheet step 6"),
    ("LINE_PARTNER_OWNER_PLAN_N_STEP7", "Partnership {0} retirement worksheet step 7"),
    ("LINE_PARTNER_OWNER_PLAN_N_STEP8", "Partnership {0} retirement worksheet step 8"),
    ("LINE_PARTNER_OWNER_PLAN_N_STEP9", "Partnership {0} retirement worksheet step 9"),
    ("LINE_PARTNER_OWNER_PLAN_N_STEP10", "Partnership {0} retirement worksheet step 10"),
    ("LINE_PARTNER_OWNER_PLAN_N_STEP11", "Partnership {0} retirement worksheet step 11"),
    ("LINE_PARTNER_OWNER_PLAN_N_STEP12", "Partnership {0} retirement worksheet step 12"),
    ("LINE_PARTNER_OWNER_PLAN_N_STEP13", "Partnership {0} retirement worksheet step 13"),
    ("LINE_PARTNER_OWNER_PLAN_N_STEP14", "Partnership {0} retirement worksheet step 14"),
    ("LINE_PARTNER_OWNER_PLAN_N_STEP15", "Partnership {0} retirement worksheet step 15"),
    ("LINE_PARTNER_OWNER_PLAN_N_STEP16", "Partnership {0} retirement worksheet step 16"),
    ("LINE_PARTNER_OWNER_PLAN_N_STEP17", "Partnership {0} retirement worksheet step 17"),
    ("LINE_PARTNER_OWNER_PLAN_N_STEP18", "Partnership {0} retirement worksheet step 18"),
    ("LINE_PARTNER_OWNER_PLAN_N_STEP19", "Partnership {0} retirement worksheet step 19"),
    ("LINE_PARTNER_OWNER_PLAN_N_STEP20", "Partnership {0} retirement worksheet step 20"),
    ("LINE_PARTNER_OWNER_PLAN_N_STEP21", "Partnership {0} retirement worksheet step 21"),
    (
        "LINE_BUSINESS_N_EXPENSE_LINE19_PENSION_PLANS",
        "Business {0} employee pension and profit-sharing plans",
    ),
    (
        "LINE_BUSINESS_N_LEASED_VEHICLE_N_PAYMENTS",
        "Business {0} leased vehicle {1} business payments",
    ),
    (
        "LINE_BUSINESS_N_LEASED_VEHICLE_N_INCLUSION",
        "Business {0} leased vehicle {1} inclusion amount",
    ),
    ("LINE_BUSINESS_N_INVENTORY_LINE35", "Business {0} inventory line 35"),
    ("LINE_BUSINESS_N_INVENTORY_LINE36", "Business {0} inventory line 36"),
    ("LINE_BUSINESS_N_INVENTORY_LINE37", "Business {0} inventory line 37"),
    ("LINE_BUSINESS_N_INVENTORY_LINE38", "Business {0} inventory line 38"),
    ("LINE_BUSINESS_N_INVENTORY_LINE39", "Business {0} inventory line 39"),
    ("LINE_BUSINESS_N_INVENTORY_LINE40", "Business {0} inventory line 40"),
    ("LINE_BUSINESS_N_INVENTORY_LINE41", "Business {0} inventory line 41"),
    ("LINE_BUSINESS_N_INVENTORY_LINE42", "Business {0} inventory line 42"),
    ("LINE_OWNER_PLAN_N_STEP1", "Business {0} owner retirement worksheet step 1"),
    ("LINE_OWNER_PLAN_N_STEP2", "Business {0} owner retirement worksheet step 2"),
    ("LINE_OWNER_PLAN_N_STEP3", "Business {0} owner retirement worksheet step 3"),
    ("LINE_OWNER_PLAN_N_STEP5", "Business {0} owner retirement worksheet step 5"),
    ("LINE_OWNER_PLAN_N_STEP6", "Business {0} owner retirement worksheet step 6"),
    ("LINE_OWNER_PLAN_N_STEP7", "Business {0} owner retirement worksheet step 7"),
    ("LINE_OWNER_PLAN_N_STEP8", "Business {0} owner retirement worksheet step 8"),
    ("LINE_OWNER_PLAN_N_STEP9", "Business {0} owner retirement worksheet step 9"),
    ("LINE_OWNER_PLAN_N_STEP10", "Business {0} owner retirement worksheet step 10"),
    ("LINE_OWNER_PLAN_N_STEP11", "Business {0} owner retirement worksheet step 11"),
    ("LINE_OWNER_PLAN_N_STEP12", "Business {0} owner retirement worksheet step 12"),
    ("LINE_OWNER_PLAN_N_STEP13", "Business {0} owner retirement worksheet step 13"),
    ("LINE_OWNER_PLAN_N_STEP14", "Business {0} owner retirement worksheet step 14"),
    ("LINE_OWNER_PLAN_N_STEP15", "Business {0} owner retirement worksheet step 15"),
    ("LINE_OWNER_PLAN_N_STEP16", "Business {0} owner retirement worksheet step 16"),
    ("LINE_OWNER_PLAN_N_STEP17", "Business {0} owner retirement worksheet step 17"),
    ("LINE_OWNER_PLAN_N_STEP18", "Business {0} owner retirement worksheet step 18"),
    ("LINE_OWNER_PLAN_N_STEP19", "Business {0} owner retirement worksheet step 19"),
    ("LINE_OWNER_PLAN_N_STEP20", "Business {0} owner retirement worksheet step 20"),
    ("LINE_OWNER_PLAN_N_STEP21", "Business {0} owner retirement worksheet step 21"),
    ("LINE_EMPLOYEE_EXPENSES_N_LEASE_N_LINE_24A", "Employee expense form {0} lease {1} line 24a"),
    ("LINE_EMPLOYEE_EXPENSES_N_LEASE_N_LINE_24B", "Employee expense form {0} lease {1} line 24b"),
    ("LINE_EMPLOYEE_EXPENSES_N_LEASE_N_LINE_24C", "Employee expense form {0} lease {1} line 24c"),
    ("LINE_EMPLOYEE_EXPENSES_N_LEASE_N_LINE_N", "Employee expense form {0} lease {1} line {2}"),
    ("DISPLAY_BUSINESS_ADDITIONAL", "Tips, overtime and fuel tax"),
    ("LINE_BUSINESS_N_GROSS_INCOME", "Business {0} gross income"),
    ("LINE_BUSINESS_N_GROSS_RECEIPTS", "Business {0} gross receipts"),
    ("LINE_BUSINESS_N_HEALTH_INSURANCE", "Business {0} health insurance"),
    ("LINE_BUSINESS_N_HOME_OFFICE", "Business {0} home office"),
    ("LINE_BUSINESS_N_INTANGIBLE_N_AMORTIZATION", "Business {0} intangible {1} amortization"),
    (
        "LINE_BUSINESS_N_INTANGIBLE_N_PERIOD",
        "Business {0} intangible {1} amortization period in months",
    ),
    (
        "LINE_BUSINESS_N_INTANGIBLE_N_AMORTIZABLE",
        "Business {0} intangible {1} cost being amortized",
    ),
    ("LINE_BUSINESS_N_INTANGIBLE_N_REMAINING_BASIS", "Business {0} intangible {1} remaining basis"),
    ("LINE_BUSINESS_N_KIDDIE_EARNED_INCOME", "Business {0} kiddie earned income"),
    ("LINE_BUSINESS_N_LONG_TERM_CARE_DEDUCTION", "Business {0} long-term care deduction"),
    ("LINE_BUSINESS_N_MACRS_MID_QUARTER", "Business {0} MACRS mid quarter"),
    ("LINE_BUSINESS_N_MARKETPLACE_HEALTH_INSURANCE", "Business {0} marketplace health insurance"),
    ("LINE_BUSINESS_N_MEDICAL_PREMIUMS", "Business {0} medical premiums"),
    ("LINE_BUSINESS_N_NET_PROFIT", "Business {0} net profit"),
    ("LINE_BUSINESS_N_OFFICE_DEPRECIATION_CARRY", "Business {0} office depreciation carry"),
    ("LINE_BUSINESS_N_OFFICE_OPERATING_CARRY", "Business {0} office operating carry"),
    ("LINE_BUSINESS_N_OTHER_INCOME", "Business {0} other income"),
    ("LINE_BUSINESS_N_QBI", "Business {0} QBI"),
    ("LINE_BUSINESS_N_QBI_LIMITED", "Business {0} QBI limited component"),
    ("LINE_BUSINESS_N_QBI_PATRON_REDUCTION", "Business {0} QBI patron reduction"),
    ("LINE_BUSINESS_N_QBI_COMPONENT", "Business {0} QBI component"),
    ("LINE_BUSINESS_N_QBI_TWENTY_PERCENT", "Business {0} QBI twenty percent"),
    ("LINE_BUSINESS_N_QUALIFIED_OVERTIME", "Business {0} qualified overtime"),
    ("LINE_BUSINESS_N_QUALIFIED_TIPS", "Business {0} qualified tips"),
    ("LINE_BUSINESS_N_SECTION179_DEDUCTION", "Business {0} section 179 deduction"),
    ("LINE_BUSINESS_N_SEP_DEDUCTION", "Business {0} SEP deduction"),
    ("LINE_BUSINESS_N_SEP_LIMIT", "Business {0} SEP limit"),
    ("LINE_BUSINESS_N_STARTUP_AMORTIZATION", "Business {0} startup amortization"),
    ("LINE_BUSINESS_N_STARTUP_PERIOD", "Business {0} start-up amortization period in months"),
    ("LINE_BUSINESS_N_STARTUP_AMORTIZABLE", "Business {0} start-up costs being amortized"),
    ("LINE_BUSINESS_N_FORM_4562B_TOTAL", "Business {0} total amortization for 2026"),
    ("LINE_BUSINESS_N_FORM_4562B_EARLIER", "Business {0} amortization that began before 2026"),
    ("LINE_BUSINESS_N_STARTUP_INITIAL", "Business {0} startup initial"),
    ("LINE_BUSINESS_N_STARTUP_REMAINING_BASIS", "Business {0} startup remaining basis"),
    ("LINE_BUSINESS_N_TENTATIVE_PROFIT", "Business {0} tentative profit"),
    ("LINE_BUSINESS_N_TOTAL_EXPENSES", "Business {0} total expenses"),
    ("LINE_BUSINESS_N_VEHICLE_N_BASIS_REDUCTION", "Business {0} vehicle {1} basis reduction"),
    ("LINE_BUSINESS_N_VEHICLE_N_EXPENSE", "Business {0} vehicle {1} expense"),
    ("LINE_BUSINESS_N_VEHICLE_LOAN_INTEREST", "Business {0} vehicle loan interest"),
    ("LINE_BUSINESS_N_WAGERING_DEDUCTION", "Business {0} wagering deduction"),
    ("LINE_FORM8582_LINE_1A", "Rental real estate with active participation, income"),
    ("LINE_FORM8582_LINE_1B", "Rental real estate with active participation, loss"),
    (
        "LINE_FORM8582_LINE_1C",
        "Rental real estate with active participation, prior-years' unallowed losses",
    ),
    ("LINE_FORM8582_LINE_2A", "All other passive activities, income"),
    ("LINE_FORM8582_LINE_2B", "All other passive activities, loss"),
    ("LINE_FORM8582_LINE_2C", "All other passive activities, prior-years' unallowed losses"),
    (
        "LINE_FORM8582_PART_IX_N_ORDINARY_UNALLOWED",
        "Form 8582 Part IX activity {0}: ordinary loss, unallowed",
    ),
    (
        "LINE_FORM8582_PART_IX_N_ORDINARY_ALLOWED",
        "Form 8582 Part IX activity {0}: ordinary loss, allowed",
    ),
    (
        "LINE_FORM8582_PART_IX_N_SECTION1231_UNALLOWED",
        "Form 8582 Part IX activity {0}: section 1231 loss, unallowed",
    ),
    (
        "LINE_FORM8582_PART_IX_N_SECTION1231_ALLOWED",
        "Form 8582 Part IX activity {0}: section 1231 loss, allowed",
    ),
    (
        "LINE_FORM8582_PART_IX_N_SHORT_PROPERTY_ALLOWED",
        "Form 8582 Part IX activity {0}: short-held property loss, allowed",
    ),
    (
        "LINE_FORM8582_PART_IX_N_SHORT_PROPERTY_UNALLOWED",
        "Form 8582 Part IX activity {0}: short-held property loss, unallowed",
    ),
    ("LINE_FORM8582_LINE_1D", "Rental real estate with active participation, net"),
    ("LINE_FORM8582_LINE_2D", "All other passive activities, net"),
    ("LINE_FORM8582_LINE_N", "Passive activity line {0}"),
    ("LINE_FORM8582CR_LINE_3A", "Low-income housing credit for 2026"),
    ("LINE_FORM8582CR_LINE_3B", "Low-income housing credit from prior years"),
    ("LINE_FORM8582CR_LINE_3C", "Low-income housing credit, total"),
    ("LINE_FORM8582CR_LINE_4A", "Other passive credits for 2026"),
    ("LINE_FORM8582CR_LINE_4B", "Other passive credits from prior years"),
    ("LINE_FORM8582CR_LINE_4C", "Other passive credits, total"),
    ("LINE_FORM8582CR_LINE_N", "Passive credit line {0}"),
    ("LINE_BUSINESS_NONQUALIFIED_EXCESS_LOSS", "Business nonqualified excess loss"),
    ("LINE_BUSINESS_QUALIFIED_EXCESS_LOSS", "Business qualified excess loss"),
    ("LINE_CAPITAL_GAIN", "Capital gain"),
    ("LINE_CAPITAL_GAIN_1A", "Capital gain 1a"),
    ("LINE_CAPITAL_GAIN_1B", "Capital gain 1b"),
    ("LINE_CAPITAL_GAIN_8A", "Capital gain 8a"),
    ("LINE_CAPITAL_GAIN_8B", "Capital gain 8b"),
    ("LINE_CAPITAL_GAIN_N", "Capital gain {0}"),
    ("LINE_CAPITAL_LONG_CARRYFORWARD", "Capital long carryforward"),
    ("LINE_CAPITAL_LOSS_CARRYFORWARD", "Capital loss carryforward"),
    ("LINE_CAPITAL_SHORT_CARRYFORWARD", "Capital short carryforward"),
    ("LINE_CARE_BENEFITS_EXCLUDED", "Care benefits excluded"),
    ("LINE_CARE_BENEFITS_TAXABLE", "Care benefits taxable"),
    ("LINE_CARE_CREDIT", "Care credit"),
    ("LINE_CARE_EARNED_SPOUSE", "Care credit: spouse’s earned income"),
    ("LINE_CARE_EARNED_TAXPAYER", "Care credit: your earned income"),
    ("LINE_CASUALTY_LIVING_INSURANCE_N_TAXABLE", "Casualty living insurance {0} taxable"),
    (
        "LINE_CASUALTY_LIVING_INSURANCE_N_TEMPORARY_INCREASE",
        "Casualty living insurance {0} temporary increase",
    ),
    ("LINE_CASUALTY_LIVING_INSURANCE_N_WITHHOLDING", "Casualty living insurance {0} withholding"),
    ("LINE_CHAPTER_ONE_NET_TAX", "Chapter one net tax"),
    ("LINE_CHARITY_N_CARRYFORWARD", "Charity {0} carryforward"),
    ("LINE_CHARITY_N_NOL_REDUCTION", "Charity {0} NOL reduction"),
    ("LINE_CHARITY_CASH_DEDUCTION", "Charity cash deduction"),
    ("LINE_CHARITY_CURRENT_CASH_N_CARRYFORWARD", "Charity current cash {0} carryforward"),
    ("LINE_CHARITY_FLOOR_NOT_CARRIED", "Charity floor not carried"),
    ("LINE_CHARITY_HOSTED_STUDENT_N_CARRYFORWARD", "Charity hosted student {0} carryforward"),
    ("LINE_CHARITY_HOSTED_STUDENT_N_ELIGIBLE", "Charity hosted student {0} eligible"),
    ("LINE_CHARITY_HOSTED_STUDENT_DEDUCTION", "Charity hosted student deduction"),
    ("LINE_CHARITY_NONCASH_N_CARRYFORWARD", "Charity noncash {0} carryforward"),
    ("LINE_CHARITY_NONCASH_DEDUCTION", "Charity noncash deduction"),
    ("LINE_CHARITY_OTHER_N_PERCENT_N_CARRYFORWARD", "Charity other {0} percent {1} carryforward"),
    ("LINE_CHARITY_OTHER_N_PERCENT_DEDUCTION", "Charity other {0} percent deduction"),
    ("LINE_CHARITY_PRIOR_CASH_N_UNUSED", "Charity prior cash {0} unused"),
    ("LINE_CHARITY_PRIOR_CASH_N_USED", "Charity prior cash {0} used"),
    (
        "LINE_CHARITY_PRIOR_CASH_CONSUMED_WITHOUT_DEDUCTION",
        "Charity prior cash consumed without deduction",
    ),
    ("LINE_CHARITY_PRIOR_CASH_DEDUCTION", "Charity prior cash deduction"),
    ("LINE_CHARITY_PRIOR_CASH_EXPIRED_AFTER_N", "Charity prior cash expired after {0}"),
    ("LINE_CHARITY_PRIOR_CASH_FLOOR_NOT_CARRIED", "Charity prior cash floor not carried"),
    ("LINE_CHARITY_PRIOR_CASH_REMAINING_N", "Charity prior cash remaining {0}"),
    (
        "LINE_CHARITY_PRIOR_HOUSEHOLD_N_CONSUMED_WITHOUT_DEDUCTION",
        "Charity prior household {0} consumed without deduction",
    ),
    ("LINE_CHARITY_PRIOR_HOUSEHOLD_N_DEDUCTION", "Charity prior household {0} deduction"),
    (
        "LINE_CHARITY_PRIOR_HOUSEHOLD_N_FLOOR_NOT_CARRIED",
        "Charity prior household {0} floor not carried",
    ),
    ("LINE_CHARITY_PRIOR_HOUSEHOLD_N_OPENING", "Charity prior household {0} opening"),
    ("LINE_CHARITY_PRIOR_HOUSEHOLD_N_REMAINING_N", "Charity prior household {0} remaining {1}"),
    ("LINE_CHARITY_PRIOR_HOUSEHOLD_N_USED", "Charity prior household {0} used"),
    (
        "LINE_CHARITY_PRIOR_STOCK_N_CONSUMED_WITHOUT_DEDUCTION",
        "Charity prior stock {0} consumed without deduction",
    ),
    ("LINE_CHARITY_PRIOR_STOCK_N_DEDUCTION", "Charity prior stock {0} deduction"),
    ("LINE_CHARITY_PRIOR_STOCK_N_FLOOR_NOT_CARRIED", "Charity prior stock {0} floor not carried"),
    ("LINE_CHARITY_PRIOR_STOCK_N_OPENING", "Charity prior stock {0} opening"),
    ("LINE_CHARITY_PRIOR_STOCK_N_ORIGINAL_BASIS", "Charity prior stock {0} original basis"),
    ("LINE_CHARITY_PRIOR_STOCK_N_ORIGINAL_VALUE", "Charity prior stock {0} original value"),
    ("LINE_CHARITY_PRIOR_STOCK_N_REMAINING_N", "Charity prior stock {0} remaining {1}"),
    ("LINE_CHARITY_PRIOR_STOCK_N_USED", "Charity prior stock {0} used"),
    ("LINE_CHARITY_PUBLIC_STOCK_N_CARRYFORWARD", "Charity public stock {0} carryforward"),
    ("LINE_CHARITY_PUBLIC_STOCK_N_CLAIM", "Charity public stock {0} claim"),
    ("LINE_CHARITY_PUBLIC_STOCK_N_PERCENT_DEDUCTION", "Charity public stock {0} percent deduction"),
    ("LINE_CHARITY_VOLUNTEER_N_EXPENSE", "Charity volunteer {0} expense"),
    ("LINE_CHARITY_WHALING_N_CARRYFORWARD", "Charity whaling {0} carryforward"),
    ("LINE_CHARITY_WHALING_DEDUCTION", "Charity whaling deduction"),
    ("LINE_CHARITY_WHALING_ELIGIBLE_EXPENSES", "Charity whaling eligible expenses"),
    ("LINE_CHARITY_WHALING_SPOUSE_ELIGIBLE_EXPENSES", "Charity whaling eligible expenses, spouse"),
    ("LINE_CHARITY_WHALING_SPOUSE_STATUTORY_CLAIM", "Charity whaling statutory claim, spouse"),
    ("LINE_CHARITY_WHALING_STATUTORY_CLAIM", "Charity whaling statutory claim"),
    (
        "LINE_CHARITY_WHALING_TAXPAYER_ELIGIBLE_EXPENSES",
        "Charity whaling eligible expenses, taxpayer",
    ),
    ("LINE_CHARITY_WHALING_TAXPAYER_STATUTORY_CLAIM", "Charity whaling statutory claim, taxpayer"),
    ("LINE_CHILD_ANNUITY_N_DEFERRED_EXCLUSION", "Child annuity {0} deferred exclusion"),
    ("LINE_CHILD_ANNUITY_N_INCOME_EXCLUSION", "Child annuity {0} income exclusion"),
    ("LINE_CHILD_ANNUITY_N_REMAINING_COST", "Child annuity {0} remaining cost"),
    ("LINE_CHILD_CREDIT", "Child credit"),
    ("LINE_CIVIL_SERVICE_DEATH_N_COST_RECOVERED", "Civil service death {0} cost recovered"),
    ("LINE_CIVIL_SERVICE_DEATH_N_TAXABLE", "Civil service death {0} taxable"),
    ("LINE_CIVIL_SERVICE_DEATH_N_UNRECOVERED_COST", "Civil service death {0} unrecovered cost"),
    ("LINE_CLEAN_VEHICLE_N_BUSINESS_PART", "Clean vehicle {0} business credit"),
    ("LINE_CLEAN_VEHICLE_N_POTENTIAL", "Clean vehicle {0} potential"),
    ("LINE_CLEAN_VEHICLE_MODIFIED_AGI_N", "Clean vehicle modified AGI {0}"),
    ("LINE_COOPERATIVE_N_DISTRIBUTIONS", "Cooperative {0} distributions"),
    ("LINE_COOPERATIVE_N_NONQUALIFIED", "Cooperative {0} nonqualified"),
    ("LINE_COOPERATIVE_N_PATRON_REDUCTION", "Cooperative {0} patron reduction"),
    ("LINE_COOPERATIVE_N_TAXABLE", "Cooperative {0} taxable"),
    ("LINE_CURRENT_NET_OPERATING_LOSS", "Current net operating loss"),
    ("LINE_CURRENT_NOL_DEBT_REDUCTION", "Current NOL debt reduction"),
    ("LINE_DC_HOMEBUYER_CARRYFORWARD_N", "DC homebuyer carryforward {0}"),
    ("LINE_DC_HOMEBUYER_CREDIT", "DC homebuyer credit"),
    ("LINE_DEBT_N_AMT_BASIS_REDUCTION", "Debt {0} AMT basis reduction"),
    ("LINE_DEBT_N_AMT_CAPITAL_AFTER", "Debt {0} AMT capital after"),
    ("LINE_DEBT_N_AMT_CAPITAL_REDUCTION", "Debt {0} AMT capital reduction"),
    ("LINE_DEBT_N_AMT_ELECTION_BASIS_REDUCTION", "Debt {0} AMT election basis reduction"),
    ("LINE_DEBT_N_AMT_FARM_BASIS_REDUCTION", "Debt {0} AMT farm basis reduction"),
    ("LINE_DEBT_N_AMT_FTC_AFTER", "Debt {0} AMT FTC after"),
    ("LINE_DEBT_N_AMT_FTC_REDUCTION", "Debt {0} AMT FTC reduction"),
    ("LINE_DEBT_N_AMT_NOL_REDUCTION", "Debt {0} AMT NOL reduction"),
    ("LINE_DEBT_N_AMT_PASSIVE_AFTER", "Debt {0} AMT passive after"),
    ("LINE_DEBT_N_AMT_PASSIVE_REDUCTION", "Debt {0} AMT passive reduction"),
    ("LINE_DEBT_N_AMT_PROPERTY_N_REDUCTION", "Debt {0} AMT property {1} reduction"),
    ("LINE_DEBT_N_AMT_PROPERTY_N_REMAINING_BASIS", "Debt {0} AMT property {1} remaining basis"),
    ("LINE_DEBT_N_AMT_REAL_PROPERTY_BASIS_REDUCTION", "Debt {0} AMT real property basis reduction"),
    ("LINE_DEBT_N_BANKRUPTCY_EXCLUSION", "Debt {0} bankruptcy exclusion"),
    ("LINE_DEBT_N_BASIS_REDUCTION", "Debt {0} basis reduction"),
    ("LINE_DEBT_N_CAPITAL_AFTER", "Debt {0} capital after"),
    ("LINE_DEBT_N_CAPITAL_REDUCTION", "Debt {0} capital reduction"),
    ("LINE_DEBT_N_ELECTION_BASIS_REDUCTION", "Debt {0} election basis reduction"),
    ("LINE_DEBT_N_FARM_BASIS_REDUCTION", "Debt {0} farm basis reduction"),
    ("LINE_DEBT_N_FARM_EXCLUSION", "Debt {0} farm exclusion"),
    ("LINE_DEBT_N_FORM982_LINE10A", "Debt {0} Form 982 line 10a"),
    ("LINE_DEBT_N_FORM982_LINE11A", "Debt {0} Form 982 line 11a"),
    ("LINE_DEBT_N_FORM982_LINE12", "Debt {0} Form 982 line 12"),
    ("LINE_DEBT_N_FORM982_LINE13", "Debt {0} Form 982 line 13"),
    ("LINE_DEBT_N_FORM982_LINE1A", "Debt {0} Form 982 line 1a"),
    ("LINE_DEBT_N_FORM982_LINE1B", "Debt {0} Form 982 line 1b"),
    ("LINE_DEBT_N_FORM982_LINE1C", "Debt {0} Form 982 line 1c"),
    ("LINE_DEBT_N_FORM982_LINE1D", "Debt {0} Form 982 line 1d"),
    ("LINE_DEBT_N_FORM982_LINE2", "Debt {0} Form 982 line 2"),
    ("LINE_DEBT_N_FORM982_LINE4", "Debt {0} Form 982 line 4"),
    ("LINE_DEBT_N_FORM982_LINE5", "Debt {0} Form 982 line 5"),
    ("LINE_DEBT_N_FORM982_LINE6", "Debt {0} Form 982 line 6"),
    ("LINE_DEBT_N_FORM982_LINE8", "Debt {0} Form 982 line 8"),
    ("LINE_DEBT_N_FORM982_LINE9", "Debt {0} Form 982 line 9"),
    ("LINE_DEBT_N_FTC_AFTER", "Debt {0} FTC after"),
    ("LINE_DEBT_N_FTC_REDUCTION", "Debt {0} FTC reduction"),
    ("LINE_DEBT_N_INSOLVENCY_EXCLUSION", "Debt {0} insolvency exclusion"),
    ("LINE_DEBT_N_MTC_AFTER", "Debt {0} MTC after (minimum tax credit)"),
    ("LINE_DEBT_N_MTC_REDUCTION", "Debt {0} MTC reduction (minimum tax credit)"),
    ("LINE_DEBT_N_PASSIVE_AFTER", "Debt {0} passive after"),
    ("LINE_DEBT_N_PASSIVE_REDUCTION", "Debt {0} passive reduction"),
    ("LINE_DEBT_N_PRICE_REDUCTION", "Debt {0} price reduction"),
    ("LINE_DEBT_N_PROPERTY_N_REDUCTION", "Debt {0} property {1} reduction"),
    ("LINE_DEBT_N_PROPERTY_N_REMAINING_BASIS", "Debt {0} property {1} remaining basis"),
    ("LINE_DEBT_N_REAL_PROPERTY_BASIS_REDUCTION", "Debt {0} real property basis reduction"),
    ("LINE_DEBT_N_REAL_PROPERTY_EXCLUSION", "Debt {0} real property exclusion"),
    ("LINE_DEBT_N_REGULAR_NOL_REDUCTION", "Debt {0} regular NOL reduction"),
    ("LINE_DEBT_N_REMAINING_BASIS", "Debt {0} remaining basis"),
    ("LINE_DEBT_N_SALE_GAIN", "Debt {0} sale gain"),
    ("LINE_DEBT_N_STUDENT_EXCLUSION", "Debt {0} student exclusion"),
    ("LINE_DEBT_N_TRANSFER_AMOUNT_REALIZED", "Debt {0} transfer amount realized"),
    ("LINE_DEBT_N_TRANSFER_GAIN", "Debt {0} transfer gain"),
    ("LINE_DEBT_CANCELLATION_N_TAXABLE", "Debt cancellation {0} taxable"),
    ("LINE_DEDUCTION", "Deduction"),
    ("LINE_DISASTER_MITIGATION_N_EXCLUDED", "Disaster mitigation {0} excluded"),
    ("LINE_DISASTER_RELIEF_N_EXCLUDED", "Disaster relief {0} excluded"),
    ("LINE_DISASTER_RELIEF_N_WITHHOLDING", "Disaster relief {0} withholding"),
    ("LINE_DISTRIBUTION_N_BASIS_REDUCTION", "Distribution {0} basis reduction"),
    ("LINE_DISTRIBUTION_N_CAPITAL_GAIN", "Distribution {0} capital gain"),
    ("LINE_DISTRIBUTION_N_PROPERTY_N_BASIS", "Distribution {0} property {1} basis"),
    ("LINE_DISTRIBUTION_N_REMAINING_BASIS", "Distribution {0} remaining basis"),
    ("LINE_DOMESTIC_ABUSE_USED_SPOUSE", "Domestic abuse used, spouse"),
    ("LINE_DOMESTIC_ABUSE_USED_TAXPAYER", "Domestic abuse used, taxpayer"),
    ("LINE_DONATED_VEHICLE_N_CLAIM", "Donated vehicle {0} claim"),
    ("LINE_EARLY_TAXPAYER_EXCEPTIONS", "Early distributions, taxpayer, exceptions"),
    ("LINE_EARLY_SPOUSE_EXCEPTIONS", "Early distributions, spouse, exceptions"),
    ("LINE_EARLY_TAXPAYER_SUBJECT", "Early distributions, taxpayer, subject"),
    ("LINE_EARLY_SPOUSE_SUBJECT", "Early distributions, spouse, subject"),
    ("LINE_EARLY_TAXPAYER_TAX", "Early distributions, taxpayer, tax"),
    ("LINE_EARLY_SPOUSE_TAX", "Early distributions, spouse, tax"),
    ("LINE_EARLY_TAXPAYER_TAXABLE", "Early distributions, taxpayer, taxable"),
    ("LINE_EARLY_SPOUSE_TAXABLE", "Early distributions, spouse, taxable"),
    ("LINE_EARLY_DISTRIBUTION_TAX", "Early distribution tax"),
    ("LINE_EARNED_INCOME_CREDIT", "Earned income credit"),
    ("LINE_EDUCATION_CREDIT", "Education credit"),
    ("LINE_EDUCATION_N_SCHOLARSHIP_INCLUDED", "Education {0} scholarship included in income"),
    ("LINE_EDUCATION_CREDIT_RECAPTURE_TAX", "Education credit recapture tax"),
    ("LINE_EIC_INVESTMENT_INCOME", "EIC investment income"),
    ("LINE_ELDERLY_CREDIT", "Elderly credit"),
    ("LINE_EMPLOYEE_STOCK_N_BASIS", "Employee stock {0} basis"),
    ("LINE_EMPLOYEE_STOCK_N_CAPITAL_GAIN", "Employee stock {0} capital gain"),
    ("LINE_EMPLOYEE_STOCK_N_COMPENSATION", "Employee stock {0} compensation"),
    ("LINE_EMPLOYEE_TIP_INCOME", "Employee tip income"),
    ("LINE_EMPLOYEE_TIPS_N_MEDICARE_TAX", "Employee tips {0} Medicare tax"),
    ("LINE_EMPLOYEE_TIPS_N_SOCIAL_TAX", "Employee tips {0} social tax"),
    ("LINE_ADDITIONAL_MEDICARE_OTHER_EMPLOYEE_WAGES", "Other actual Medicare-covered employee pay"),
    (
        "LINE_EMPLOYEE_COVERAGE_N_SOCIAL_TAX",
        "Employee {0} known coverage worksheet Social Security tax",
    ),
    ("LINE_EMPLOYEE_COVERAGE_N_MEDICARE_TAX", "Employee {0} known coverage worksheet Medicare tax"),
    ("LINE_EMPLOYEE_COVERAGE_N_TAX", "Employee {0} known coverage worksheet payroll subtotal"),
    ("LINE_EMPLOYEE_WAGES_N_MEDICARE_TAX", "Employee wages {0} Medicare tax"),
    ("LINE_EMPLOYEE_WAGES_N_SOCIAL_TAX", "Employee wages {0} social tax"),
    ("LINE_EMPLOYER_LOAN_INTEREST_REMOVED", "Employer loan interest removed"),
    ("LINE_ESPP_SALE_N_BASIS", "ESPP sale {0} basis"),
    ("LINE_ESPP_SALE_N_CAPITAL_GAIN", "ESPP sale {0} capital gain"),
    ("LINE_ESPP_SALE_N_COMPENSATION", "ESPP sale {0} compensation"),
    ("LINE_ESTATE_PAID_MEDICAL_ELECTION", "Estate paid medical election"),
    ("LINE_ESTATE_TRUST_67E_DEDUCTION", "Estate trust 67e deduction"),
    (
        "LINE_ESTATE_TRUST_N_ACTIVITY_N_33C",
        "Estate/trust {0}, activity {1}: allowed passive losses",
    ),
    ("LINE_ESTATE_TRUST_N_ACTIVITY_N_33D", "Estate/trust {0}, activity {1}: passive income"),
    ("LINE_ESTATE_TRUST_N_ACTIVITY_N_33E", "Estate/trust {0}, activity {1}: nonpassive deductions"),
    ("LINE_ESTATE_TRUST_N_ACTIVITY_N_33F", "Estate/trust {0}, activity {1}: nonpassive income"),
    (
        "LINE_ESTATE_TRUST_N_ACTIVITY_N_SUSPENDED",
        "Estate/trust {0}, activity {1}: passive loss carried forward",
    ),
    (
        "LINE_AMT_ESTATE_TRUST_N_ACTIVITY_N_SUSPENDED",
        "Estate/trust {0}, activity {1}: AMT passive loss carried forward",
    ),
    ("LINE_ESTATE_TRUST_N_11A_67E_DEDUCTION", "Estate trust {0} 11a 67e deduction"),
    ("LINE_ESTATE_TRUST_N_11B_PERSONAL_PROPERTY_TAX", "Estate trust {0} 11b personal property tax"),
    ("LINE_ESTATE_TRUST_N_11B_REAL_ESTATE_TAX", "Estate trust {0} 11b real estate tax"),
    ("LINE_ESTATE_TRUST_N_11B_STATE_INCOME_TAX", "Estate trust {0} 11b state income tax"),
    ("LINE_ESTATE_TRUST_N_11B_STATE_SALES_TAX", "Estate trust {0} 11b state sales tax"),
    ("LINE_ESTATE_TRUST_N_11C_SHORT_CARRY", "Estate trust {0} 11c short carry"),
    ("LINE_ESTATE_TRUST_N_11D_LONG_CARRY", "Estate trust {0} 11d long carry"),
    ("LINE_ESTATE_TRUST_N_11E_REGULAR_NOL", "Estate trust {0} 11e regular NOL"),
    ("LINE_ESTATE_TRUST_N_11F_AMT_NOL", "Estate trust {0} 11f AMT NOL"),
    ("LINE_ESTATE_TRUST_N_12A_AMT_ADJUSTMENT", "Estate trust {0} 12a AMT adjustment"),
    (
        "LINE_ESTATE_TRUST_N_12B_AMT_QUALIFIED_DIVIDENDS",
        "Estate trust {0} 12b AMT qualified dividends",
    ),
    ("LINE_ESTATE_TRUST_N_12C_AMT_SHORT_GAIN", "Estate trust {0} 12c AMT short gain"),
    ("LINE_ESTATE_TRUST_N_12D_AMT_LONG_GAIN", "Estate trust {0} 12d AMT long gain"),
    ("LINE_ESTATE_TRUST_N_12E_AMT_UNRECAPTURED_GAIN", "Estate trust {0} 12e AMT unrecaptured gain"),
    ("LINE_ESTATE_TRUST_N_12F_AMT_RATE_N_GAIN", "Estate trust {0} 12f AMT rate {1} gain"),
    ("LINE_ESTATE_TRUST_N_14A_PRIVATE_ACTIVITY", "Estate trust {0} 14a private activity"),
    ("LINE_ESTATE_TRUST_N_14A_TAX_EXEMPT_INTEREST", "Estate trust {0} 14a tax exempt interest"),
    (
        "LINE_ESTATE_TRUST_N_14F_GROSS_FARMING_FISHING_INCOME",
        "Estate trust {0} 14f gross farming fishing income",
    ),
    ("LINE_ESTATE_TRUST_N_14H_NIIT_LINE_N", "Estate trust {0} 14h NIIT line {1}"),
    ("LINE_ESTATE_TRUST_N_14H_NIIT_MAGI", "Estate trust {0} 14h NIIT MAGI"),
    (
        "LINE_ESTATE_TRUST_N_14I_SECTION199A_DIVIDENDS",
        "Estate trust {0} 14i section 199a dividends",
    ),
    ("LINE_ESTATE_TRUST_N_2A_ORDINARY_DIVIDENDS", "Estate trust {0} 2a ordinary dividends"),
    ("LINE_ESTATE_TRUST_N_2B_QUALIFIED_DIVIDENDS", "Estate trust {0} 2b qualified dividends"),
    ("LINE_ESTATE_TRUST_N_4A_LONG_GAIN", "Estate trust {0} 4a long gain"),
    ("LINE_ESTATE_TRUST_N_4B_RATE_N_GAIN", "Estate trust {0} 4b rate {1} gain"),
    ("LINE_ESTATE_TRUST_N_4C_UNRECAPTURED_GAIN", "Estate trust {0} 4c unrecaptured gain"),
    ("LINE_ESTATE_TRUST_N_9A_ROYALTY_DEPRECIATION", "Estate trust {0} 9a royalty depreciation"),
    ("LINE_ESTATE_TRUST_N_9B_ROYALTY_DEPLETION", "Estate trust {0} 9b royalty depletion"),
    ("LINE_ESTATE_TRUST_N_9C_ROYALTY_AMORTIZATION", "Estate trust {0} 9c royalty amortization"),
    ("LINE_ESTATE_TRUST_N_LINE33F_ROYALTIES", "Estate trust {0} line 33f royalties"),
    (
        "LINE_ESTATE_TRUST_N_N_BUSINESS_APPORTIONED_DEDUCTIONS",
        "Estate trust {0} {1} business apportioned deductions",
    ),
    ("LINE_ESTATE_TRUST_N_N_BUSINESS_INCOME", "Estate trust {0} {1} business income"),
    ("LINE_ESTATE_TRUST_N_N_ESTATE_TAX", "Estate trust {0} {1} estate tax"),
    ("LINE_ESTATE_TRUST_N_N_INTEREST", "Estate trust {0} {1} interest"),
    (
        "LINE_ESTATE_TRUST_N_N_OTHER_RENTAL_APPORTIONED_DEDUCTIONS",
        "Estate trust {0} {1} other rental apportioned deductions",
    ),
    ("LINE_ESTATE_TRUST_N_N_OTHER_RENTAL_INCOME", "Estate trust {0} {1} other rental income"),
    (
        "LINE_ESTATE_TRUST_N_N_RENTAL_REAL_ESTATE_APPORTIONED_DEDUCTIONS",
        "Estate trust {0} {1} rental real estate apportioned deductions",
    ),
    (
        "LINE_ESTATE_TRUST_N_N_RENTAL_REAL_ESTATE_INCOME",
        "Estate trust {0} {1} rental real estate income",
    ),
    ("LINE_ESTATE_TRUST_N_N_ROYALTY_INCOME", "Estate trust {0} {1} royalty income"),
    ("LINE_ESTATE_TRUST_N_N_SHORT_GAIN", "Estate trust {0} {1} short gain"),
    ("LINE_ESTATE_TRUST_N_ALLOCATED_ESTIMATED_TAX", "Estate trust {0} allocated estimated tax"),
    (
        "LINE_ESTATE_TRUST_N_ATNOL_N_CARRYFORWARD",
        "Estate trust {0} ATNOL {1} carryforward (alternative tax net operating loss)",
    ),
    (
        "LINE_ESTATE_TRUST_N_ATNOL_N_DEBT_REDUCTION",
        "Estate trust {0} ATNOL {1} debt reduction (alternative tax net operating loss)",
    ),
    (
        "LINE_ESTATE_TRUST_N_ATNOL_N_EXPIRED",
        "Estate trust {0} ATNOL {1} expired (alternative tax net operating loss)",
    ),
    (
        "LINE_ESTATE_TRUST_N_ATNOL_N_OPENING",
        "Estate trust {0} ATNOL {1} opening (alternative tax net operating loss)",
    ),
    (
        "LINE_ESTATE_TRUST_N_ATNOL_N_USED",
        "Estate trust {0} ATNOL {1} used (alternative tax net operating loss)",
    ),
    ("LINE_ESTATE_TRUST_N_BACKUP_WITHHOLDING", "Estate trust {0} backup withholding"),
    ("LINE_ESTATE_TRUST_N_NOL_N_CARRYFORWARD", "Estate trust {0} NOL {1} carryforward"),
    ("LINE_ESTATE_TRUST_N_NOL_N_DEBT_REDUCTION", "Estate trust {0} NOL {1} debt reduction"),
    ("LINE_ESTATE_TRUST_N_NOL_N_EXPIRED", "Estate trust {0} NOL {1} expired"),
    ("LINE_ESTATE_TRUST_N_NOL_N_OPENING", "Estate trust {0} NOL {1} opening"),
    ("LINE_ESTATE_TRUST_N_NOL_N_USED", "Estate trust {0} NOL {1} used"),
    ("LINE_ESTATE_TRUST_N_PASSIVE_ACTIVITY_INCOME", "Estate trust {0} passive activity income"),
    (
        "LINE_ESTATE_TRUST_IRD_ESTATE_TAX_DEDUCTION",
        "Estate trust IRD estate tax deduction (income in respect of a decedent)",
    ),
    ("LINE_EXCESS_BUSINESS_LOSS", "Excess business loss"),
    ("LINE_EXCESS_BUSINESS_LOSS_CARRYFORWARD", "Excess business loss carryforward"),
    ("LINE_FAMILY_LEAVE_1099G", "Family leave 1099-G"),
    ("LINE_FARM_N_ACCRUAL_SALES", "Farm {0} accrual sales"),
    ("LINE_FARM_N_CCC_ADVANCES", "Farm {0} CCC advances (Commodity Credit Corporation)"),
    ("LINE_FARM_N_CCC_FORFEITURE", "Farm {0} CCC forfeiture (Commodity Credit Corporation)"),
    (
        "LINE_FARM_N_CCC_FORFEITURE_REPORTED",
        "Farm {0} CCC forfeiture reported (Commodity Credit Corporation)",
    ),
    ("LINE_FARM_N_CCC_INCOME", "Farm {0} CCC income (Commodity Credit Corporation)"),
    ("LINE_FARM_N_CCC_MARKET_GAIN", "Farm {0} CCC market gain (Commodity Credit Corporation)"),
    (
        "LINE_FARM_N_CCC_PAID_DEFICIENCY",
        "Farm {0} CCC paid deficiency (Commodity Credit Corporation)",
    ),
    (
        "LINE_FARM_N_CCC_RETAINED_BASIS",
        "Farm {0} CCC retained basis (Commodity Credit Corporation)",
    ),
    ("LINE_FARM_N_CCC_SALES_NET", "Farm {0} CCC sales net (Commodity Credit Corporation)"),
    ("LINE_FARM_N_CCC_SURPLUS", "Farm {0} CCC surplus (Commodity Credit Corporation)"),
    (
        "LINE_FARM_N_CCC_TAXABLE_MARKET_GAIN",
        "Farm {0} CCC taxable market gain (Commodity Credit Corporation)",
    ),
    ("LINE_FARM_N_CCC_WITHHOLDING", "Farm {0} CCC withholding (Commodity Credit Corporation)"),
    ("LINE_FARM_N_CLOSING_INVENTORY", "Farm {0} closing inventory"),
    ("LINE_FARM_N_CONSERVATION_ASSESSMENT_AVAILABLE", "Farm {0} conservation assessment available"),
    ("LINE_FARM_N_CONSERVATION_ASSESSMENT_DEFERRED", "Farm {0} conservation assessment deferred"),
    ("LINE_FARM_N_CONSERVATION_BASIS_ADDITION", "Farm {0} conservation basis addition"),
    ("LINE_FARM_N_CONSERVATION_CARRYFORWARD", "Farm {0} conservation carryforward"),
    ("LINE_FARM_N_CONSERVATION_DEDUCTION", "Farm {0} conservation deduction"),
    ("LINE_FARM_N_CONSERVATION_EXPIRED", "Farm {0} conservation expired"),
    ("LINE_FARM_N_CONSERVATION_INCOME_LIMIT", "Farm {0} conservation income limit"),
    ("LINE_FARM_N_COOPERATIVE_DISTRIBUTIONS", "Farm {0} cooperative distributions"),
    ("LINE_FARM_N_COOPERATIVE_TAXABLE", "Farm {0} cooperative taxable"),
    ("LINE_FARM_N_CROP_INSURANCE_BEFORE_DEFERRAL", "Farm {0} crop insurance before deferral"),
    ("LINE_FARM_N_CROP_INSURANCE_CURRENT_TAXABLE", "Farm {0} crop insurance current taxable"),
    ("LINE_FARM_N_CROP_INSURANCE_DEFERRED_N", "Farm {0} crop insurance deferred {1}"),
    ("LINE_FARM_N_CROP_INSURANCE_PRIOR_INCLUDED", "Farm {0} crop insurance prior included"),
    ("LINE_FARM_N_CROP_INSURANCE_TAXABLE", "Farm {0} crop insurance taxable"),
    ("LINE_FARM_N_CROP_INSURANCE_WITHHOLDING", "Farm {0} crop insurance withholding"),
    ("LINE_FARM_N_CRP_INCOME", "Farm {0} CRP income (Conservation Reserve Program)"),
    ("LINE_FARM_N_CRP_SE_EXCLUSION", "Farm {0} CRP SE exclusion (Conservation Reserve Program)"),
    ("LINE_FARM_N_CRP_WITHHOLDING", "Farm {0} CRP withholding (Conservation Reserve Program)"),
    ("LINE_FARM_N_DEPRECIATION", "Farm {0} depreciation"),
    ("LINE_FARM_N_EXPENSE_LINE11_CHEMICALS", "Farm {0} expense line 11 chemicals"),
    ("LINE_FARM_N_EXPENSE_LINE13_CUSTOM_HIRE", "Farm {0} expense line 13 custom hire"),
    (
        "LINE_FARM_N_EXPENSE_LINE15_EMPLOYEE_BENEFIT_PROGRAMS",
        "Farm {0} expense line 15 employee benefit programs",
    ),
    ("LINE_FARM_N_EXPENSE_LINE16_FEED", "Farm {0} expense line 16 feed"),
    (
        "LINE_FARM_N_EXPENSE_LINE17_FERTILIZERS_AND_LIME",
        "Farm {0} expense line 17 fertilizers and lime",
    ),
    (
        "LINE_FARM_N_EXPENSE_LINE18_FREIGHT_AND_TRUCKING",
        "Farm {0} expense line 18 freight and trucking",
    ),
    (
        "LINE_FARM_N_EXPENSE_LINE19_GASOLINE_FUEL_AND_OIL",
        "Farm {0} expense line 19 gasoline fuel and oil",
    ),
    ("LINE_FARM_N_EXPENSE_LINE20_INSURANCE", "Farm {0} expense line 20 insurance"),
    (
        "LINE_FARM_N_EXPENSE_LINE21A_MORTGAGE_INTEREST",
        "Farm {0} expense line 21a mortgage interest",
    ),
    ("LINE_FARM_N_EXPENSE_LINE21C_OTHER_INTEREST", "Farm {0} expense line 21c other interest"),
    ("LINE_FARM_N_EXPENSE_LINE22_LABOR_HIRED", "Farm {0} expense line 22 labor hired"),
    (
        "LINE_FARM_N_EXPENSE_LINE23_PENSION_AND_PROFIT_SHARING",
        "Farm {0} expense line 23 pension and profit sharing",
    ),
    (
        "LINE_FARM_N_EXPENSE_LINE24A_RENT_VEHICLES_EQUIPMENT",
        "Farm {0} expense line 24a rent vehicles equipment",
    ),
    ("LINE_FARM_N_EXPENSE_LINE24B_RENT_OTHER", "Farm {0} expense line 24b rent other"),
    ("LINE_FARM_N_EXPENSE_LINE25_REPAIRS", "Farm {0} expense line 25 repairs"),
    ("LINE_FARM_N_EXPENSE_LINE26_SEEDS_AND_PLANTS", "Farm {0} expense line 26 seeds and plants"),
    (
        "LINE_FARM_N_EXPENSE_LINE27_STORAGE_AND_WAREHOUSING",
        "Farm {0} expense line 27 storage and warehousing",
    ),
    ("LINE_FARM_N_EXPENSE_LINE28_SUPPLIES", "Farm {0} expense line 28 supplies"),
    ("LINE_FARM_N_EXPENSE_LINE29_TAXES", "Farm {0} expense line 29 taxes"),
    ("LINE_FARM_N_EXPENSE_LINE30_UTILITIES", "Farm {0} expense line 30 utilities"),
    (
        "LINE_FARM_N_EXPENSE_LINE31_VETERINARY_BREEDING_AND_MEDICINE",
        "Farm {0} expense line 31 veterinary breeding and medicine",
    ),
    ("LINE_FARM_N_EXPENSE_LINE32_ADVERTISING", "Farm {0} advertising"),
    ("LINE_FARM_N_EXPENSE_LINE32_COMMISSIONS_AND_FEES", "Farm {0} commissions and fees"),
    (
        "LINE_FARM_N_EXPENSE_LINE32_LEGAL_AND_PROFESSIONAL",
        "Farm {0} legal and professional services",
    ),
    ("LINE_FARM_N_EXPENSE_LINE32_OFFICE", "Farm {0} office expenses"),
    ("LINE_FARM_N_EXPENSE_LINE32_MEALS", "Farm {0} expense line 32 meals"),
    (
        "LINE_FARM_N_EXPENSE_LINE32_STARTUP_AMORTIZATION",
        "Farm {0} expense lines 32a to 32f start-up costs and amortization",
    ),
    (
        "LINE_FARM_N_EXPENSE_LINE32_PREPAID_SUPPLIES",
        "Farm {0} expense lines 32a to 32f prepaid supplies",
    ),
    (
        "LINE_FARM_N_EXPENSE_LINE32_CCC_DEFICIENCY",
        "Farm {0} expense lines 32a to 32f CCC loan deficiency paid",
    ),
    ("LINE_FARM_N_EXPENSE_LINE32_OTHER", "Farm {0} expense line 32 other"),
    ("LINE_FARM_N_EXPENSES", "Farm {0} expenses"),
    ("LINE_FARM_N_GROSS_INCOME", "Farm {0} gross income"),
    ("LINE_FARM_N_LAND_N_CONSERVATION_RECAPTURE", "Farm {0} land {1} conservation recapture"),
    ("LINE_FARM_N_LAND_N_FORM6252_ADJUSTED_BASIS", "Farm {0} land {1} Form 6252 adjusted basis"),
    ("LINE_FARM_N_LAND_N_FORM6252_ASSUMED_DEBT", "Farm {0} land {1} Form 6252 assumed debt"),
    ("LINE_FARM_N_LAND_N_FORM6252_CONTRACT_PRICE", "Farm {0} land {1} Form 6252 contract price"),
    ("LINE_FARM_N_LAND_N_FORM6252_COST", "Farm {0} land {1} Form 6252 cost"),
    ("LINE_FARM_N_LAND_N_FORM6252_DEEMED_PAYMENT", "Farm {0} land {1} Form 6252 deemed payment"),
    ("LINE_FARM_N_LAND_N_FORM6252_DEPRECIATION", "Farm {0} land {1} Form 6252 depreciation"),
    ("LINE_FARM_N_LAND_N_FORM6252_EXCESS_DEBT", "Farm {0} land {1} Form 6252 excess debt"),
    ("LINE_FARM_N_LAND_N_FORM6252_EXPENSES", "Farm {0} land {1} Form 6252 expenses"),
    ("LINE_FARM_N_LAND_N_FORM6252_GAIN", "Farm {0} land {1} Form 6252 gain"),
    ("LINE_FARM_N_LAND_N_FORM6252_GROSS_PROFIT", "Farm {0} land {1} Form 6252 gross profit"),
    ("LINE_FARM_N_LAND_N_FORM6252_PROFIT", "Farm {0} land {1} Form 6252 profit before exclusion"),
    (
        "LINE_FARM_N_LAND_N_FORM6252_PERCENTAGE",
        "Farm {0} land {1} Form 6252 gross profit percentage",
    ),
    ("LINE_FARM_N_LAND_N_FORM6252_INCOME", "Farm {0} land {1} Form 6252 income"),
    ("LINE_FARM_N_LAND_N_FORM6252_NET_PRICE", "Farm {0} land {1} Form 6252 net price"),
    ("LINE_FARM_N_LAND_N_FORM6252_ORDINARY", "Farm {0} land {1} Form 6252 ordinary"),
    ("LINE_FARM_N_LAND_N_FORM6252_PAYMENTS", "Farm {0} land {1} Form 6252 payments"),
    ("LINE_FARM_N_LAND_N_FORM6252_PRICE", "Farm {0} land {1} Form 6252 price"),
    ("LINE_FARM_N_LAND_N_FORM6252_RECAPTURE", "Farm {0} land {1} Form 6252 recapture"),
    ("LINE_FARM_N_LAND_N_FORM6252_TOTAL", "Farm {0} land {1} Form 6252 total"),
    ("LINE_FARM_N_LAND_N_FORM6252_TOTAL_PAYMENTS", "Farm {0} land {1} Form 6252 total payments"),
    ("LINE_FARM_N_LAND_N_GAIN", "Farm {0} land {1} gain"),
    ("LINE_FARM_N_LAND_N_ORDINARY", "Farm {0} land {1} ordinary"),
    ("LINE_FARM_N_LAND_N_SECTION1231", "Farm {0} land {1} section 1231"),
    ("LINE_FARM_N_NET_PROFIT", "Farm {0} net profit"),
    ("LINE_FARM_N_OPENING_INVENTORY", "Farm {0} opening inventory"),
    ("LINE_FARM_N_OTHER_ACCRUAL_INCOME", "Farm {0} other accrual income"),
    ("LINE_FARM_N_PRODUCT_PURCHASES", "Farm {0} product purchases"),
    ("LINE_FARM_N_PROGRAM_PAYMENTS", "Farm {0} program payments"),
    ("LINE_FARM_N_PROGRAM_TAXABLE", "Farm {0} program taxable"),
    ("LINE_FARM_N_RAISED_SALES", "Farm {0} raised sales"),
    ("LINE_FARM_N_RESALE_BASIS", "Farm {0} resale basis"),
    ("LINE_FARM_N_RESALE_PROCEEDS", "Farm {0} resale proceeds"),
    ("LINE_FARM_N_SUPPLIES_DEDUCTION", "Farm {0} supplies deduction"),
    ("LINE_FARM_N_SUPPLIES_ELIGIBLE", "Farm {0} supplies eligible"),
    ("LINE_FARM_N_SUPPLIES_POULTRY_DISPOSED_COST", "Farm {0} supplies poultry disposed cost"),
    (
        "LINE_FARM_N_SUPPLIES_PRODUCTION_POULTRY_RELEASED",
        "Farm {0} supplies production poultry released",
    ),
    ("LINE_FARM_N_SUPPLIES_RELEASED", "Farm {0} supplies released"),
    ("LINE_FARM_N_SUPPLIES_REMAINING", "Farm {0} supplies remaining"),
    ("LINE_FARM_AVERAGING_N_ALLOCATED_INCOME", "Farm averaging {0} allocated income"),
    ("LINE_FARM_AVERAGING_N_BASE_INCOME", "Farm averaging {0} base income"),
    ("LINE_FARM_AVERAGING_N_BASE_TAX", "Farm averaging {0} base tax"),
    ("LINE_FARM_AVERAGING_N_FOREIGN_DIVIDENDS", "Farm averaging {0} foreign dividends"),
    ("LINE_FARM_AVERAGING_N_FOREIGN_GAIN_EXCESS", "Farm averaging {0} foreign gain excess"),
    ("LINE_FARM_AVERAGING_N_FOREIGN_LINE18", "Farm averaging {0} foreign line 18"),
    ("LINE_FARM_AVERAGING_N_FOREIGN_LINE19", "Farm averaging {0} foreign line 19"),
    ("LINE_FARM_AVERAGING_N_FOREIGN_NET_GAIN", "Farm averaging {0} foreign net gain"),
    ("LINE_FARM_AVERAGING_N_FOREIGN_STACKING", "Farm averaging {0} foreign stacking"),
    ("LINE_FARM_AVERAGING_N_INVESTMENT_CAPITAL", "Farm averaging {0} investment capital"),
    ("LINE_FARM_AVERAGING_N_INVESTMENT_DIVIDENDS", "Farm averaging {0} investment dividends"),
    ("LINE_FARM_AVERAGING_N_RECOMPUTED_TAX", "Farm averaging {0} recomputed tax"),
    ("LINE_FARM_AVERAGING_CURRENT_TAX", "Farm averaging current tax"),
    ("LINE_FARM_AVERAGING_EACH_YEAR", "Farm averaging each year"),
    ("LINE_FARM_AVERAGING_ELECTED_INCOME", "Farm averaging elected income"),
    ("LINE_FARM_AVERAGING_ELIGIBLE_INCOME", "Farm averaging eligible income"),
    ("LINE_FARM_AVERAGING_QBI_DEDUCTION", "Farm averaging QBI deduction"),
    ("LINE_FARM_AVERAGING_REMAINING_INCOME", "Farm averaging remaining income"),
    ("LINE_FARM_AVERAGING_TAX", "Farm averaging tax"),
    ("LINE_FARM_AVERAGING_WITHOUT_ELECTION", "Farm averaging without election"),
    ("LINE_FARM_INSTALLMENT_ACTUAL_CREDITS", "Farm installment actual credits"),
    ("LINE_FARM_INSTALLMENT_ACTUAL_FOREIGN_CREDIT", "Farm installment actual foreign credit"),
    ("LINE_FARM_INSTALLMENT_ACTUAL_REGULAR_LIABILITY", "Farm installment actual regular liability"),
    ("LINE_FARM_INSTALLMENT_APPLICABLE", "Farm installment applicable"),
    (
        "LINE_FARM_INSTALLMENT_AVERAGING_N_ALLOCATED_INCOME",
        "Farm installment averaging {0} allocated income",
    ),
    (
        "LINE_FARM_INSTALLMENT_AVERAGING_N_FOREIGN_DIVIDENDS",
        "Farm installment averaging {0} foreign dividends",
    ),
    (
        "LINE_FARM_INSTALLMENT_AVERAGING_N_FOREIGN_GAIN_EXCESS",
        "Farm installment averaging {0} foreign gain excess",
    ),
    (
        "LINE_FARM_INSTALLMENT_AVERAGING_N_FOREIGN_LINE18",
        "Farm installment averaging {0} foreign line 18",
    ),
    (
        "LINE_FARM_INSTALLMENT_AVERAGING_N_FOREIGN_LINE19",
        "Farm installment averaging {0} foreign line 19",
    ),
    (
        "LINE_FARM_INSTALLMENT_AVERAGING_N_FOREIGN_NET_GAIN",
        "Farm installment averaging {0} foreign net gain",
    ),
    (
        "LINE_FARM_INSTALLMENT_AVERAGING_N_FOREIGN_STACKING",
        "Farm installment averaging {0} foreign stacking",
    ),
    (
        "LINE_FARM_INSTALLMENT_AVERAGING_N_RECOMPUTED_TAX",
        "Farm installment averaging {0} recomputed tax",
    ),
    ("LINE_FARM_INSTALLMENT_AVERAGING_ELECTED_INCOME", "Farm installment averaging elected income"),
    ("LINE_FARM_INSTALLMENT_CURRENT", "Farm installment current"),
    ("LINE_FARM_INSTALLMENT_ELECTED_GAIN", "Farm installment elected gain"),
    ("LINE_FARM_INSTALLMENT_FIRST", "Farm installment first"),
    ("LINE_FARM_INSTALLMENT_REDUCED_CREDITS", "Farm installment reduced credits"),
    ("LINE_FARM_INSTALLMENT_REDUCED_FOREIGN_CREDIT", "Farm installment reduced foreign credit"),
    (
        "LINE_FARM_INSTALLMENT_REDUCED_REGULAR_LIABILITY",
        "Farm installment reduced regular liability",
    ),
    ("LINE_FARM_INSTALLMENT_REDUCED_REGULAR_TAX", "Farm installment reduced regular tax"),
    ("LINE_FARM_INSTALLMENT_REDUCED_TAXABLE", "Farm installment reduced taxable"),
    ("LINE_FARM_INSTALLMENT_REGULAR_ADDITIONS", "Farm installment regular additions"),
    ("LINE_FARM_INSTALLMENT_REMAINING", "Farm installment remaining"),
    ("LINE_FARMING_FISHING_GROSS_INCOME_N", "Farming fishing gross income {0}"),
    ("LINE_FINAL_BOND_INTEREST_N", "Final bond interest {0}"),
    ("LINE_FINAL_BOND_RECIPIENT_INTEREST_N", "Final bond recipient interest {0}"),
    ("LINE_FIRST_HOME_USED_TAXPAYER", "First home used, taxpayer"),
    ("LINE_FOREIGN_AMT_DOMESTIC_DEDUCTIONS", "Foreign AMT domestic deductions"),
    (
        "LINE_FOREIGN_AMT_GENERAL_OVERALL_DOMESTIC_LOSS_NEW",
        "Foreign AMT general overall domestic loss new",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_OVERALL_DOMESTIC_LOSS_RECAPTURED",
        "Foreign AMT general overall domestic loss recaptured",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_OVERALL_DOMESTIC_LOSS_REMAINING",
        "Foreign AMT general overall domestic loss remaining",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_OVERALL_FOREIGN_LOSS_RECAPTURED",
        "Foreign AMT general overall foreign loss recaptured",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_OVERALL_FOREIGN_LOSS_REMAINING",
        "Foreign AMT general overall foreign loss remaining",
    ),
    ("LINE_FOREIGN_AMT_GENERAL_SEPARATE_LOSS_NETTED", "Foreign AMT general separate loss netted"),
    (
        "LINE_FOREIGN_AMT_GENERAL_SEPARATE_LOSS_RECAPTURED",
        "Foreign AMT general separate loss recaptured",
    ),
    (
        "LINE_FOREIGN_AMT_GENERAL_SEPARATE_LOSS_REMAINING",
        "Foreign AMT general separate loss remaining",
    ),
    ("LINE_FOREIGN_AMT_HIGH_TAX_NET_INCOME", "Foreign AMT high tax net income"),
    ("LINE_FOREIGN_AMT_HIGH_TAX_TAXES", "Foreign AMT high tax taxes"),
    ("LINE_FOREIGN_AMT_LINE_N_RATE_ADJUSTMENT", "Foreign AMT line {0} rate adjustment"),
    ("LINE_FOREIGN_AMT_PASSIVE_LOSS_REALLOCATED", "Foreign AMT passive loss reallocated"),
    (
        "LINE_FOREIGN_AMT_PASSIVE_OVERALL_DOMESTIC_LOSS_NEW",
        "Foreign AMT passive overall domestic loss new",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_OVERALL_DOMESTIC_LOSS_RECAPTURED",
        "Foreign AMT passive overall domestic loss recaptured",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_OVERALL_DOMESTIC_LOSS_REMAINING",
        "Foreign AMT passive overall domestic loss remaining",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_OVERALL_FOREIGN_LOSS_NEW",
        "Foreign AMT passive overall foreign loss new",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_OVERALL_FOREIGN_LOSS_RECAPTURED",
        "Foreign AMT passive overall foreign loss recaptured",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_OVERALL_FOREIGN_LOSS_REMAINING",
        "Foreign AMT passive overall foreign loss remaining",
    ),
    ("LINE_FOREIGN_AMT_PASSIVE_SEPARATE_LOSS_NETTED", "Foreign AMT passive separate loss netted"),
    ("LINE_FOREIGN_AMT_PASSIVE_SEPARATE_LOSS_NEW", "Foreign AMT passive separate loss new"),
    (
        "LINE_FOREIGN_AMT_PASSIVE_SEPARATE_LOSS_RECAPTURED",
        "Foreign AMT passive separate loss recaptured",
    ),
    (
        "LINE_FOREIGN_AMT_PASSIVE_SEPARATE_LOSS_REMAINING",
        "Foreign AMT passive separate loss remaining",
    ),
    ("LINE_FOREIGN_AMT_QUALIFIED_DIVIDEND_ADJUSTMENT", "Foreign AMT qualified dividend adjustment"),
    (
        "LINE_FOREIGN_BUSINESS_SPOUSE_DISALLOWED_DEDUCTIONS",
        "Foreign business disallowed deductions, spouse",
    ),
    ("LINE_FOREIGN_BUSINESS_SPOUSE_GROSS_EXCLUSION", "Foreign business gross exclusion, spouse"),
    ("LINE_FOREIGN_BUSINESS_SPOUSE_NET_EXCLUSION", "Foreign business net exclusion, spouse"),
    (
        "LINE_FOREIGN_BUSINESS_TAXPAYER_DISALLOWED_DEDUCTIONS",
        "Foreign business disallowed deductions, taxpayer",
    ),
    (
        "LINE_FOREIGN_BUSINESS_TAXPAYER_GROSS_EXCLUSION",
        "Foreign business gross exclusion, taxpayer",
    ),
    ("LINE_FOREIGN_BUSINESS_TAXPAYER_NET_EXCLUSION", "Foreign business net exclusion, taxpayer"),
    ("LINE_FOREIGN_EARNED_EXCLUSION", "Foreign earned exclusion"),
    ("LINE_FOREIGN_EARNED_TAXPAYER_INCOME", "Foreign earned income, taxpayer"),
    ("LINE_FOREIGN_EARNED_SPOUSE_INCOME", "Foreign earned income, spouse"),
    ("LINE_FOREIGN_EARNED_TAXPAYER_DAYS", "Foreign qualifying days, taxpayer"),
    ("LINE_FOREIGN_EARNED_SPOUSE_DAYS", "Foreign qualifying days, spouse"),
    ("LINE_FOREIGN_GENERAL_EXCLUDED_TAX_REDUCTION", "Foreign general tax on excluded income"),
    (
        "LINE_FOREIGN_GENERAL_AMT_EXCLUDED_TAX_REDUCTION",
        "Foreign general AMT tax on excluded income",
    ),
    ("LINE_FOREIGN_EMPLOYEE_SPOUSE_GROSS_EXCLUSION", "Foreign employee gross exclusion, spouse"),
    (
        "LINE_FOREIGN_EMPLOYEE_TAXPAYER_GROSS_EXCLUSION",
        "Foreign employee gross exclusion, taxpayer",
    ),
    ("LINE_FOREIGN_GENERAL_AMT_N_CARRY_REMAINING", "Foreign general AMT {0} carry remaining"),
    ("LINE_FOREIGN_GENERAL_AMT_N_CARRY_USED", "Foreign general AMT {0} carry used"),
    ("LINE_FOREIGN_GENERAL_AMT_CARRY_EXPIRED", "Foreign general AMT carry expired"),
    ("LINE_FOREIGN_GENERAL_AMT_CARRY_USED", "Foreign general AMT carry used"),
    ("LINE_FOREIGN_GENERAL_AMT_CARRYBACK_N", "Foreign general AMT carryback {0}"),
    (
        "LINE_FOREIGN_GENERAL_AMT_DEDUCTION_YEAR_ABSORBED_N",
        "Foreign general AMT deduction year absorbed {0}",
    ),
    ("LINE_FOREIGN_GENERAL_AMT_NEW_CARRYFORWARD", "Foreign general AMT new carryforward"),
    ("LINE_FOREIGN_GENERAL_AMT_RETAINED_CARRYFORWARD", "Foreign general AMT retained carryforward"),
    ("LINE_FOREIGN_GENERAL_AMT_UNUSED_TAX", "Foreign general AMT unused tax"),
    (
        "LINE_FOREIGN_GENERAL_REGULAR_N_CARRY_REMAINING",
        "Foreign general regular {0} carry remaining",
    ),
    ("LINE_FOREIGN_GENERAL_REGULAR_N_CARRY_USED", "Foreign general regular {0} carry used"),
    ("LINE_FOREIGN_GENERAL_REGULAR_CARRY_EXPIRED", "Foreign general regular carry expired"),
    ("LINE_FOREIGN_GENERAL_REGULAR_CARRY_USED", "Foreign general regular carry used"),
    ("LINE_FOREIGN_GENERAL_REGULAR_CARRYBACK_N", "Foreign general regular carryback {0}"),
    (
        "LINE_FOREIGN_GENERAL_REGULAR_DEDUCTION_YEAR_ABSORBED_N",
        "Foreign general regular deduction year absorbed {0}",
    ),
    ("LINE_FOREIGN_GENERAL_REGULAR_NEW_CARRYFORWARD", "Foreign general regular new carryforward"),
    (
        "LINE_FOREIGN_GENERAL_REGULAR_RETAINED_CARRYFORWARD",
        "Foreign general regular retained carryforward",
    ),
    ("LINE_FOREIGN_GENERAL_UNUSED_TAX", "Foreign general unused tax"),
    ("LINE_FOREIGN_HIGH_TAX_NET_INCOME", "Foreign high tax net income"),
    ("LINE_FOREIGN_HIGH_TAX_TAXES", "Foreign high tax taxes"),
    ("LINE_FOREIGN_HOUSING_DEDUCTION", "Foreign housing deduction"),
    ("LINE_FOREIGN_HOUSING_EXCLUSION", "Foreign housing exclusion"),
    ("LINE_FOREIGN_HOUSING_SPOUSE_BASE", "Foreign housing base, spouse"),
    ("LINE_FOREIGN_HOUSING_SPOUSE_CURRENT_DEDUCTION", "Foreign housing current deduction, spouse"),
    (
        "LINE_FOREIGN_HOUSING_SPOUSE_DEDUCTION_CARRYFORWARD",
        "Foreign housing deduction carryforward, spouse",
    ),
    ("LINE_FOREIGN_HOUSING_SPOUSE_EXCLUSION", "Foreign housing exclusion, spouse"),
    ("LINE_FOREIGN_HOUSING_SPOUSE_EXPENSES", "Foreign housing expenses, spouse"),
    ("LINE_FOREIGN_HOUSING_SPOUSE_LIMIT", "Foreign housing limit, spouse"),
    (
        "LINE_FOREIGN_HOUSING_SPOUSE_NONTAXABLE_ALLOWANCES",
        "Foreign housing nontaxable allowances, spouse",
    ),
    (
        "LINE_FOREIGN_HOUSING_SPOUSE_PRIOR_DEDUCTION_EXPIRED",
        "Foreign housing prior deduction expired, spouse",
    ),
    (
        "LINE_FOREIGN_HOUSING_SPOUSE_PRIOR_DEDUCTION_USED",
        "Foreign housing prior deduction used, spouse",
    ),
    ("LINE_FOREIGN_HOUSING_TAXPAYER_BASE", "Foreign housing base, taxpayer"),
    (
        "LINE_FOREIGN_HOUSING_TAXPAYER_CURRENT_DEDUCTION",
        "Foreign housing current deduction, taxpayer",
    ),
    (
        "LINE_FOREIGN_HOUSING_TAXPAYER_DEDUCTION_CARRYFORWARD",
        "Foreign housing deduction carryforward, taxpayer",
    ),
    ("LINE_FOREIGN_HOUSING_TAXPAYER_EXCLUSION", "Foreign housing exclusion, taxpayer"),
    ("LINE_FOREIGN_HOUSING_TAXPAYER_EXPENSES", "Foreign housing expenses, taxpayer"),
    ("LINE_FOREIGN_HOUSING_TAXPAYER_LIMIT", "Foreign housing limit, taxpayer"),
    (
        "LINE_FOREIGN_HOUSING_TAXPAYER_NONTAXABLE_ALLOWANCES",
        "Foreign housing nontaxable allowances, taxpayer",
    ),
    (
        "LINE_FOREIGN_HOUSING_TAXPAYER_PRIOR_DEDUCTION_EXPIRED",
        "Foreign housing prior deduction expired, taxpayer",
    ),
    (
        "LINE_FOREIGN_HOUSING_TAXPAYER_PRIOR_DEDUCTION_USED",
        "Foreign housing prior deduction used, taxpayer",
    ),
    ("LINE_FOREIGN_INVESTMENT_AMT_GROUP_N", "Foreign investment AMT group {0}"),
    ("LINE_FOREIGN_INVESTMENT_REGULAR_GROUP_N", "Foreign investment regular group {0}"),
    (
        "LINE_FOREIGN_K3_DEFINITELY_RELATED_DEDUCTIONS",
        "Foreign Schedule K-3 definitely related deductions",
    ),
    ("LINE_FOREIGN_K3_GROSS_INCOME_CORRECTION", "Foreign Schedule K-3 gross income correction"),
    ("LINE_FOREIGN_K3_INTEREST_EXPENSE", "Foreign Schedule K-3 interest expense"),
    ("LINE_FOREIGN_K3_TAX_REDUCTION", "Foreign Schedule K-3 tax reduction"),
    ("LINE_FOREIGN_LINE_N_RATE_ADJUSTMENT", "Foreign line {0} rate adjustment"),
    ("LINE_FOREIGN_PASSIVE_AMT_N_CARRY_REMAINING", "Foreign passive AMT {0} carry remaining"),
    ("LINE_FOREIGN_PASSIVE_AMT_N_CARRY_USED", "Foreign passive AMT {0} carry used"),
    ("LINE_FOREIGN_PASSIVE_AMT_CARRY_EXPIRED", "Foreign passive AMT carry expired"),
    ("LINE_FOREIGN_PASSIVE_AMT_CARRY_USED", "Foreign passive AMT carry used"),
    ("LINE_FOREIGN_PASSIVE_AMT_CARRYBACK_N", "Foreign passive AMT carryback {0}"),
    (
        "LINE_FOREIGN_PASSIVE_AMT_DEDUCTION_YEAR_ABSORBED_N",
        "Foreign passive AMT deduction year absorbed {0}",
    ),
    ("LINE_FOREIGN_PASSIVE_AMT_NEW_CARRYFORWARD", "Foreign passive AMT new carryforward"),
    ("LINE_FOREIGN_PASSIVE_AMT_RETAINED_CARRYFORWARD", "Foreign passive AMT retained carryforward"),
    ("LINE_FOREIGN_PASSIVE_AMT_UNUSED_TAX", "Foreign passive AMT unused tax"),
    (
        "LINE_FOREIGN_PASSIVE_REGULAR_N_CARRY_REMAINING",
        "Foreign passive regular {0} carry remaining",
    ),
    ("LINE_FOREIGN_PASSIVE_REGULAR_N_CARRY_USED", "Foreign passive regular {0} carry used"),
    ("LINE_FOREIGN_PASSIVE_REGULAR_CARRY_EXPIRED", "Foreign passive regular carry expired"),
    ("LINE_FOREIGN_PASSIVE_REGULAR_CARRY_USED", "Foreign passive regular carry used"),
    ("LINE_FOREIGN_PASSIVE_REGULAR_CARRYBACK_N", "Foreign passive regular carryback {0}"),
    (
        "LINE_FOREIGN_PASSIVE_REGULAR_DEDUCTION_YEAR_ABSORBED_N",
        "Foreign passive regular deduction year absorbed {0}",
    ),
    ("LINE_FOREIGN_PASSIVE_REGULAR_NEW_CARRYFORWARD", "Foreign passive regular new carryforward"),
    (
        "LINE_FOREIGN_PASSIVE_REGULAR_RETAINED_CARRYFORWARD",
        "Foreign passive regular retained carryforward",
    ),
    ("LINE_FOREIGN_PASSIVE_UNUSED_TAX", "Foreign passive unused tax"),
    ("LINE_FOREIGN_QUALIFIED_DIVIDEND_ADJUSTMENT", "Foreign qualified dividend adjustment"),
    ("LINE_FOREIGN_REGULAR_DOMESTIC_DEDUCTIONS", "Foreign regular domestic deductions"),
    (
        "LINE_FOREIGN_REGULAR_GENERAL_OVERALL_DOMESTIC_LOSS_NEW",
        "Foreign regular general overall domestic loss new",
    ),
    (
        "LINE_FOREIGN_REGULAR_GENERAL_OVERALL_DOMESTIC_LOSS_RECAPTURED",
        "Foreign regular general overall domestic loss recaptured",
    ),
    (
        "LINE_FOREIGN_REGULAR_GENERAL_OVERALL_DOMESTIC_LOSS_REMAINING",
        "Foreign regular general overall domestic loss remaining",
    ),
    (
        "LINE_FOREIGN_REGULAR_GENERAL_OVERALL_FOREIGN_LOSS_NEW",
        "Foreign regular general overall foreign loss new",
    ),
    (
        "LINE_FOREIGN_REGULAR_GENERAL_OVERALL_FOREIGN_LOSS_RECAPTURED",
        "Foreign regular general overall foreign loss recaptured",
    ),
    (
        "LINE_FOREIGN_REGULAR_GENERAL_OVERALL_FOREIGN_LOSS_REMAINING",
        "Foreign regular general overall foreign loss remaining",
    ),
    (
        "LINE_FOREIGN_REGULAR_GENERAL_SEPARATE_LOSS_NETTED",
        "Foreign regular general separate loss netted",
    ),
    ("LINE_FOREIGN_REGULAR_GENERAL_SEPARATE_LOSS_NEW", "Foreign regular general separate loss new"),
    (
        "LINE_FOREIGN_REGULAR_GENERAL_SEPARATE_LOSS_RECAPTURED",
        "Foreign regular general separate loss recaptured",
    ),
    (
        "LINE_FOREIGN_REGULAR_GENERAL_SEPARATE_LOSS_REMAINING",
        "Foreign regular general separate loss remaining",
    ),
    ("LINE_FOREIGN_REGULAR_PASSIVE_LOSS_REALLOCATED", "Foreign regular passive loss reallocated"),
    (
        "LINE_FOREIGN_REGULAR_PASSIVE_OVERALL_DOMESTIC_LOSS_NEW",
        "Foreign regular passive overall domestic loss new",
    ),
    (
        "LINE_FOREIGN_REGULAR_PASSIVE_OVERALL_DOMESTIC_LOSS_RECAPTURED",
        "Foreign regular passive overall domestic loss recaptured",
    ),
    (
        "LINE_FOREIGN_REGULAR_PASSIVE_OVERALL_DOMESTIC_LOSS_REMAINING",
        "Foreign regular passive overall domestic loss remaining",
    ),
    (
        "LINE_FOREIGN_REGULAR_PASSIVE_OVERALL_FOREIGN_LOSS_NEW",
        "Foreign regular passive overall foreign loss new",
    ),
    (
        "LINE_FOREIGN_REGULAR_PASSIVE_OVERALL_FOREIGN_LOSS_RECAPTURED",
        "Foreign regular passive overall foreign loss recaptured",
    ),
    (
        "LINE_FOREIGN_REGULAR_PASSIVE_OVERALL_FOREIGN_LOSS_REMAINING",
        "Foreign regular passive overall foreign loss remaining",
    ),
    (
        "LINE_FOREIGN_REGULAR_PASSIVE_SEPARATE_LOSS_NETTED",
        "Foreign regular passive separate loss netted",
    ),
    ("LINE_FOREIGN_REGULAR_PASSIVE_SEPARATE_LOSS_NEW", "Foreign regular passive separate loss new"),
    (
        "LINE_FOREIGN_REGULAR_PASSIVE_SEPARATE_LOSS_RECAPTURED",
        "Foreign regular passive separate loss recaptured",
    ),
    (
        "LINE_FOREIGN_REGULAR_PASSIVE_SEPARATE_LOSS_REMAINING",
        "Foreign regular passive separate loss remaining",
    ),
    ("LINE_FOREIGN_STATE_INCOME_TAX_GROUP_N", "Foreign state income tax group {0}"),
    ("LINE_FOREIGN_TAX_CREDIT", "Foreign tax credit"),
    ("LINE_FOREIGN_WAGES", "Foreign wages"),
    ("LINE_FORM4797_LINE10", "Form 4797 line 10"),
    ("LINE_FORM4797_LINE8", "Form 4797 line 8"),
    ("LINE_FORM4797_LINE9", "Form 4797 line 9"),
    ("LINE_FORM_N_ADDITIONAL_MODIFICATIONS", "Form {0} additional modifications"),
    ("LINE_FORM_N_AI_LINE_N_A", "Form {0} AI line {1} a"),
    ("LINE_FORM_N_AI_LINE_N_B", "Form {0} AI line {1} b"),
    ("LINE_FORM_N_AI_LINE_N_C", "Form {0} AI line {1} c"),
    ("LINE_FORM_N_AI_LINE_N_D", "Form {0} AI line {1} d"),
    ("LINE_FORM_N_ANNUITIES", "Form {0} annuities"),
    ("LINE_FORM_N_BUSINESS_PART", "Form {0} business credit"),
    ("LINE_FORM_N_BUSINESS_INVESTMENT_INCOME", "Form {0} business investment income"),
    ("LINE_FORM_N_BUSINESS_RENTAL_INCOME", "Form {0} business rental income"),
    ("LINE_FORM_N_CORRECTED_INCOME_TAX", "Form {0} corrected income tax"),
    ("LINE_FORM_N_DEDUCTIONS_AND_MODIFICATIONS", "Form {0} deductions and modifications"),
    ("LINE_FORM_N_EDUCATIONAL_BENEFITS", "Form {0} tax-free educational benefits"),
    ("LINE_FORM_N_ELIGIBLE_INTEREST", "Form {0} interest in proportion to the expenses"),
    ("LINE_FORM_N_ES_PLANNED_CASH_N", "Form {0}-ES planned installment {1} — remaining cash"),
    ("LINE_FORM_N_ES_LINE_11A", "Form {0} ES line 11a"),
    ("LINE_FORM_N_ES_LINE_11B", "Form {0} ES line 11b"),
    ("LINE_FORM_N_ES_LINE_11C", "Form {0} ES line 11c"),
    ("LINE_FORM_N_ES_LINE_12A", "Form {0} ES line 12a"),
    ("LINE_FORM_N_ES_LINE_12B", "Form {0} ES line 12b"),
    ("LINE_FORM_N_ES_LINE_12C", "Form {0} ES line 12c"),
    ("LINE_FORM_N_ES_LINE_14A", "Form {0} ES line 14a"),
    ("LINE_FORM_N_ES_LINE_14B", "Form {0} ES line 14b"),
    ("LINE_FORM_N_ES_LINE_2A", "Form {0} ES line 2a"),
    ("LINE_FORM_N_ES_LINE_2B", "Form {0} ES line 2b"),
    ("LINE_FORM_N_ES_LINE_2C", "Form {0} ES line 2c"),
    ("LINE_FORM_N_ES_LINE_2D", "Form {0} ES line 2d"),
    ("LINE_FORM_N_ES_LINE_N", "Form {0} ES line {1}"),
    ("LINE_FORM_N_EXCLUDED_GAINS", "Form {0} excluded gains"),
    ("LINE_FORM_N_F_LINE_N", "Form {0}-F line {1}"),
    ("LINE_FORM_N_F_PENALTY_BEFORE_WAIVER", "Form {0}-F penalty before the waiver"),
    ("LINE_FORM_N_F_WAIVER_REQUESTED_AMOUNT", "Form {0}-F waiver requested amount"),
    ("LINE_FORM_N_INCOME_AND_PROPERTY_TAXES", "Form {0} income and property taxes"),
    ("LINE_FORM_N_INJURED_CONTRIBUTION", "Form {0} injured contribution"),
    ("LINE_FORM_N_INJURED_EIC", "Form {0} injured EIC"),
    ("LINE_FORM_N_INJURED_LIABILITY", "Form {0} injured liability"),
    ("LINE_FORM_N_INJURED_REFUND_ESTIMATE", "Form {0} injured refund estimate"),
    ("LINE_FORM_N_INJURED_SHARE_OF_TAX", "Form {0} injured share of tax"),
    (
        "LINE_FORM_N_SUPPLEMENTAL_QBI_A",
        "Form {0} supplemental qualified business income deduction (joint)",
    ),
    (
        "LINE_FORM_N_SUPPLEMENTAL_QBI_B",
        "Form {0} supplemental qualified business income deduction (injured spouse)",
    ),
    (
        "LINE_FORM_N_SUPPLEMENTAL_QBI_C",
        "Form {0} supplemental qualified business income deduction (other spouse)",
    ),
    ("LINE_FORM_N_INTEREST", "Form {0} interest in the proceeds"),
    ("LINE_FORM_N_INVESTMENT_EXPENSES", "Form {0} investment expenses"),
    ("LINE_FORM_N_INVESTMENT_INCOME", "Form {0} investment income"),
    ("LINE_FORM_N_INVESTMENT_INTEREST", "Form {0} investment interest"),
    ("LINE_FORM_N_LINE1", "Form {0} line 1"),
    ("LINE_FORM_N_LINE11", "Form {0} line 11"),
    ("LINE_FORM_N_LINE3", "Form {0} line 3"),
    ("LINE_FORM_N_LINE4", "Form {0} line 4"),
    ("LINE_FORM8582CR_LINE_1A", "Rental real-estate credits with active participation"),
    ("LINE_FORM8582CR_LINE_1B", "Prior unallowed credits with active participation"),
    ("LINE_FORM8582CR_LINE_1C", "Total rental credits with active participation"),
    ("LINE_SCHEDULE_N_ELECTIVE_PAYMENT", "Schedule {0} elective payment"),
    ("LINE_FORM_N_PART_N_LINE_N", "Form {0} Part {1} line {2}"),
    ("LINE_FORM_N_LINE_10A", "Form {0} line 10a"),
    ("LINE_FORM_N_LINE_10C", "Form {0} line 10c"),
    ("LINE_FORM_N_LINE_11A", "Form {0} line 11a"),
    ("LINE_FORM_N_LINE_12A", "Form {0} line 12a"),
    ("LINE_FORM_N_LINE_13A", "Form {0} line 13a"),
    ("LINE_FORM_N_LINE_13A_A", "Form {0} line 13a a"),
    ("LINE_FORM_N_LINE_13A_B", "Form {0} line 13a b"),
    ("LINE_FORM_N_LINE_13A_C", "Form {0} line 13a c"),
    ("LINE_FORM_N_LINE_13B_A", "Form {0} line 13b a"),
    ("LINE_FORM_N_LINE_13B_B", "Form {0} line 13b b"),
    ("LINE_FORM_N_LINE_13B_C", "Form {0} line 13b c"),
    ("LINE_FORM_N_LINE_18A", "Form {0} line 18a"),
    ("LINE_FORM_N_LINE_19A_STUDENT_N", "Form {0} line 19a student {1}"),
    ("LINE_FORM_N_LINE_19B_STUDENT_N", "Form {0} line 19b student {1}"),
    ("LINE_FORM_N_LINE_5A", "Form {0} line 5a"),
    ("LINE_FORM_N_LINE_9A", "Form {0} line 9a"),
    ("LINE_FORM_N_LINE_N", "Form {0} line {1}"),
    ("LINE_FORM_N_LINE_N_A", "Form {0} line {1} a"),
    ("LINE_FORM_N_LINE_N_AOTC", "Form {0} line {1} AOTC"),
    ("LINE_FORM_N_LINE_N_B", "Form {0} line {1} b"),
    ("LINE_FORM_N_LINE_N_C", "Form {0} line {1} c"),
    ("LINE_FORM_N_LINE_N_CHILD_N", "Form {0} line {1} child {2}"),
    ("LINE_FORM_N_LINE_N_CHILD_N_BIRTH", "Form {0} line {1} child {2} birth"),
    ("LINE_FORM_N_LINE_N_CHILD_N_DEATH", "Form {0} line {1} child {2} death"),
    ("LINE_FORM_N_LINE_N_CHILD_CREDITS", "Form {0} line {1} child credits"),
    ("LINE_FORM_N_LINE_N_D", "Form {0} line {1} d"),
    ("LINE_FORM_N_LINE_N_EIC", "Form {0} line {1} EIC"),
    ("LINE_FORM_N_LINE_N_INTEREST", "Form {0} line {1} interest"),
    ("LINE_FORM_N_LINE_N_OTHER_DEPENDENT_N", "Form {0} line {1} other dependent {2}"),
    ("LINE_FORM_N_MISCELLANEOUS_INVESTMENT_EXPENSES", "Form {0} miscellaneous investment expenses"),
    ("LINE_FORM_N_MODIFIED_AGI", "Form {0} modified AGI"),
    ("LINE_FORM_N_MODIFIED_AGI_EXCESS", "Form {0} modified AGI excess"),
    ("LINE_FORM_N_NET_EXPENSES", "Form {0} expenses after benefits"),
    ("LINE_FORM_N_NET_GAINS", "Form {0} net gains"),
    ("LINE_FORM_N_NET_INVESTMENT_INCOME", "Form {0} net investment income"),
    ("LINE_FORM_N_NET_INVESTMENT_INCOME_TAX", "Form {0} net investment income tax"),
    ("LINE_FORM_N_NET_PAYMENT_AFTER_INTEREST", "Form {0} net payment after interest"),
    ("LINE_FORM_N_NONPASSIVE_BUSINESS_ADJUSTMENT", "Form {0} nonpassive business adjustment"),
    ("LINE_FORM_N_ORDINARY_DIVIDENDS", "Form {0} ordinary dividends"),
    ("LINE_FORM_N_OTHER_MODIFICATIONS", "Form {0} other modifications"),
    ("LINE_FORM_N_PENALTY_BEFORE_WAIVER", "Form {0} penalty before the waiver"),
    ("LINE_FORM_N_PERSONAL_GAINS", "Form {0} personal gains"),
    ("LINE_FORM_N_PERSONAL_LONG_NET", "Form {0} personal long net"),
    ("LINE_FORM_N_PERSONAL_LOSS_AFTER_AGI", "Form {0} personal loss after AGI"),
    ("LINE_FORM_N_PERSONAL_LOSS_AFTER_EVENTS", "Form {0} personal loss after events"),
    ("LINE_FORM_N_PERSONAL_NET_AFTER_GAINS", "Form {0} personal net after gains"),
    ("LINE_FORM_N_PERSONAL_SHORT_NET", "Form {0} personal short net"),
    ("LINE_FORM_N_PHASEOUT_REDUCTION", "Form {0} reduction for modified AGI"),
    ("LINE_FORM_N_PRIOR_HOUSEHOLD_N_COST", "Form {0} prior household {1} cost"),
    ("LINE_FORM_N_PRIOR_HOUSEHOLD_N_VALUE", "Form {0} prior household {1} value"),
    ("LINE_FORM_N_PRIOR_INCOME_TAX", "Form {0} prior income tax"),
    ("LINE_FORM_N_PROCEEDS", "Form {0} bond proceeds"),
    ("LINE_FORM_N_PROPERTY_GAINS", "Form {0} property gains"),
    ("LINE_FORM_N_QUALIFIED_EXPENSES", "Form {0} qualified higher education expenses"),
    ("LINE_FORM_N_SECTION_A_VALUE", "Form {0} section a value"),
    ("LINE_FORM_N_SECTION_B_VALUE", "Form {0} section b value"),
    ("LINE_FORM_N_SMALLER_AMOUNT", "Form {0} smaller amount"),
    ("LINE_FORM_N_TAXABLE_INTEREST", "Form {0} taxable interest"),
    ("LINE_FORM_N_TAXPAYER_N_EXCESS_REPAYMENT", "Form {0} {1} excess repayment, taxpayer"),
    ("LINE_FORM_N_TAXPAYER_N_INSTALLMENT_N", "Form {0} {1} installment {2}, taxpayer"),
    (
        "LINE_FORM_N_TAXPAYER_N_INSTALLMENT_FOLLOWING_YEAR",
        "Form {0} {1} installment 2027, taxpayer",
    ),
    (
        "LINE_FORM_N_TAXPAYER_N_INSTALLMENT_SECOND_FOLLOWING_YEAR",
        "Form {0} {1} installment 2028, taxpayer",
    ),
    ("LINE_FORM_N_TAXPAYER_N_LINE_N", "Form {0} {1} line {2}, taxpayer"),
    ("LINE_FORM_N_THRESHOLD", "Form {0} threshold"),
    ("LINE_FORM_N_UNDECLARED_LOSS_AFTER_EVENTS", "Form {0} undeclared loss after events"),
    ("LINE_FORM_N_UNUSABLE_DECREASE", "Form {0} unusable decrease"),
    ("LINE_FORM_N_WAIVER_REQUESTED_AMOUNT", "Form {0} waiver requested amount"),
    ("LINE_FUEL_TAX_CREDIT", "Fuel tax credit"),
    ("LINE_FUND_N_BASIS_INCREASE", "Fund {0} basis increase"),
    ("LINE_GAMBLING_LOSS_DEDUCTION", "Gambling loss deduction"),
    ("LINE_GAMBLING_WINNINGS", "Gambling winnings"),
    ("LINE_GENERAL_BUSINESS_CREDIT", "General business credit"),
    ("LINE_GENERAL_BUSINESS_CREDIT_LIMIT", "General business credit limit"),
    ("LINE_GROSS_INCOME_N", "Gross income {0}"),
    ("LINE_HEALTH_LOAN_N_EXCLUDED", "Health loan {0} excluded"),
    ("LINE_HEALTH_LOAN_INTEREST_REMOVED", "Health loan interest removed"),
    ("LINE_HOBBY_SALES_INCOME", "Hobby sales income"),
    ("LINE_HOME_N_DEPRECIATION_GAIN", "Home {0} depreciation gain"),
    ("LINE_HOME_N_EXCLUDED_GAIN", "Home {0} excluded gain"),
    (
        "LINE_HOME_N_FIRPTA_WITHHOLDING",
        "Home {0} tax withheld from a foreign seller (Form 8288-A, Form 1040 line 25c)",
    ),
    ("LINE_HOME_N_FORM6252_PRICE", "Home {0} Form 6252 line 5 selling price"),
    ("LINE_HOME_N_FORM6252_ASSUMED_DEBT", "Home {0} Form 6252 line 6 loans the buyer took over"),
    ("LINE_HOME_N_FORM6252_NET_PRICE", "Home {0} Form 6252 line 7 price less loans"),
    ("LINE_HOME_N_FORM6252_COST", "Home {0} Form 6252 line 8 cost or other basis"),
    ("LINE_HOME_N_FORM6252_DEPRECIATION", "Home {0} Form 6252 line 9 depreciation"),
    ("LINE_HOME_N_FORM6252_ADJUSTED_BASIS", "Home {0} Form 6252 line 10 adjusted basis"),
    ("LINE_HOME_N_FORM6252_EXPENSES", "Home {0} Form 6252 line 11 selling expenses"),
    ("LINE_HOME_N_FORM6252_RECAPTURE", "Home {0} Form 6252 line 12 income recapture"),
    ("LINE_HOME_N_FORM6252_TOTAL", "Home {0} Form 6252 line 13 basis, expenses and recapture"),
    ("LINE_HOME_N_FORM6252_EXCLUSION", "Home {0} Form 6252 line 15 home-sale exclusion"),
    (
        "LINE_HOME_N_FORM6252_GROSS_PROFIT",
        "Home {0} Form 6252 line 16 gross profit after the exclusion",
    ),
    ("LINE_HOME_N_FORM6252_PROFIT", "Home {0} Form 6252 line 14 profit before exclusion"),
    ("LINE_HOME_N_FORM6252_PERCENTAGE", "Home {0} Form 6252 line 19 gross profit percentage"),
    ("LINE_HOME_N_FORM6252_EXCESS_DEBT", "Home {0} Form 6252 line 17 loans above basis"),
    ("LINE_HOME_N_FORM6252_CONTRACT_PRICE", "Home {0} Form 6252 line 18 contract price"),
    (
        "LINE_HOME_N_FORM6252_DEEMED_PAYMENT",
        "Home {0} Form 6252 line 20 loans above basis as a payment",
    ),
    ("LINE_HOME_N_FORM6252_PAYMENTS", "Home {0} Form 6252 line 21 principal received in 2026"),
    ("LINE_HOME_N_FORM6252_TOTAL_PAYMENTS", "Home {0} Form 6252 line 22 total payments"),
    ("LINE_HOME_N_FORM6252_INCOME", "Home {0} Form 6252 line 24 installment sale income"),
    ("LINE_HOME_N_FORM6252_ORDINARY", "Home {0} Form 6252 line 25 ordinary income"),
    ("LINE_HOME_N_FORM6252_GAIN", "Home {0} Form 6252 line 26 gain"),
    ("LINE_HOME_N_FORM8824_LINE15", "Home {0} Form 8824 line 15 cash and other property received"),
    ("LINE_HOME_N_FORM8824_LINE16", "Home {0} Form 8824 line 16 like-kind property received"),
    ("LINE_HOME_N_FORM8824_LINE18", "Home {0} Form 8824 line 18 basis given up"),
    ("LINE_HOME_N_FORM8824_LINE19", "Home {0} Form 8824 line 19 realized gain"),
    ("LINE_HOME_N_FORM8824_LINE20", "Home {0} Form 8824 line 20 recognized gain"),
    ("LINE_HOME_N_FORM8824_LINE24", "Home {0} Form 8824 line 24 deferred gain"),
    ("LINE_HOME_N_FORM8824_LINE25", "Home {0} Form 8824 line 25 basis of the new property"),
    ("LINE_HOME_N_EXCHANGE_SECTION1231", "Home {0} exchange section 1231 gain"),
    ("LINE_HOME_N_FORM_N_ADJUSTMENT", "Home {0} Form {1} adjustment"),
    ("LINE_HOME_N_FORM_N_BASIS", "Home {0} Form {1} basis"),
    ("LINE_HOME_N_FORM_N_GAIN", "Home {0} Form {1} gain"),
    ("LINE_HOME_N_FORM_N_PROCEEDS", "Home {0} Form {1} proceeds"),
    ("LINE_HOME_N_NONQUALIFIED_GAIN", "Home {0} nonqualified gain"),
    ("LINE_HOME_N_REALIZED_GAIN", "Home {0} realized gain"),
    ("LINE_HOME_N_SEPARATE_BASIS", "Home {0} separate space adjusted basis (Form 4797 line 2)"),
    (
        "LINE_HOME_N_SEPARATE_EXPENSES",
        "Home {0} separate space selling expenses (Form 4797 line 2)",
    ),
    ("LINE_HOME_N_SEPARATE_GAIN", "Home {0} separate space gain or loss (Form 4797 line 2)"),
    ("LINE_HOME_N_SEPARATE_PRICE", "Home {0} separate space selling price (Form 4797 line 2)"),
    (
        "LINE_HOME_N_SEPARATE_UNRECAPTURED_N",
        "Home {0} separate space unrecaptured section {1} gain (Schedule D line 19)",
    ),
    ("LINE_HOME_N_TAXABLE_GAIN", "Home {0} taxable gain"),
    ("LINE_HOME_KEROSENE_FUEL_4A", "Home kerosene fuel 4a"),
    ("LINE_HOUSEHOLD_N_ADDITIONAL_MEDICARE", "Household {0} additional Medicare"),
    ("LINE_HOUSEHOLD_N_EMPLOYEE_WITHHOLDING", "Household {0} employee withholding"),
    ("LINE_HOUSEHOLD_N_FUTA_CREDIT", "Household {0} FUTA credit"),
    ("LINE_HOUSEHOLD_N_FUTA_CREDIT_REDUCTION", "Household {0} FUTA credit reduction"),
    ("LINE_HOUSEHOLD_N_FUTA_TAX", "Household {0} FUTA tax"),
    ("LINE_HOUSEHOLD_N_FUTA_WAGES", "Household {0} FUTA wages"),
    ("LINE_HOUSEHOLD_N_MEDICARE_TAX", "Household {0} Medicare tax"),
    ("LINE_HOUSEHOLD_N_MEDICARE_WAGES", "Household {0} Medicare wages"),
    ("LINE_HOUSEHOLD_N_SOCIAL_TAX", "Household {0} social tax"),
    ("LINE_HOUSEHOLD_N_SOCIAL_WAGES", "Household {0} social wages"),
    ("LINE_HOUSEHOLD_N_STATE_DISABILITY_TAX_OFFSET", "Household {0} state disability tax offset"),
    ("LINE_HOUSEHOLD_N_TAX", "Household {0} tax"),
    ("LINE_HSA_N_N_EXCESS_ACCOUNT_VALUE", "HSA {0} {1} excess account value"),
    ("LINE_HSA_N_N_UNCORRECTED_EMPLOYER_EXCESS", "HSA {0} {1} uncorrected employer excess"),
    ("LINE_HSA_N_N_UNCORRECTED_EXCESS", "HSA {0} {1} uncorrected excess"),
    ("LINE_HSA_N_CORRECTED_CONTRIBUTIONS", "HSA {0} corrected contributions"),
    (
        "LINE_HSA_N_DISTRIBUTION_ESTIMATE_INCOME",
        "HSA owner {0}: rounded distribution income in estimate",
    ),
    ("LINE_HSA_N_DISTRIBUTION_ESTIMATE_TAX", "HSA owner {0}: rounded distribution tax in estimate"),
    ("LINE_HSA_N_DISTRIBUTION_EXCEPTION", "Form 8889 owner {0}: distribution exception applies"),
    (
        "LINE_HSA_N_DISTRIBUTION_EXCLUDED",
        "Form 8889 owner {0}: rollover and timely returned excess",
    ),
    ("LINE_HSA_N_DISTRIBUTION_GROSS", "Form 8889 owner {0}: total distributions"),
    ("LINE_HSA_N_DISTRIBUTION_MEDICAL", "Form 8889 owner {0}: qualified medical spending"),
    ("LINE_HSA_N_DISTRIBUTION_NET", "Form 8889 owner {0}: distributions after exclusions"),
    (
        "LINE_HSA_N_DISTRIBUTION_PENALTY_BASE",
        "Form 8889 owner {0}: distributions subject to 20% tax",
    ),
    ("LINE_HSA_N_DISTRIBUTION_TAX", "Form 8889 owner {0}: 20% tax before estimate rounding"),
    (
        "LINE_HSA_N_DISTRIBUTION_TAXABLE",
        "Form 8889 owner {0}: taxable distributions before estimate rounding",
    ),
    ("LINE_HSA_N_PLEDGED_INCOME", "HSA owner {0}: pledged assets included directly in income"),
    ("LINE_HSA_N_PLEDGED_PENALTY_BASE", "HSA owner {0}: pledged assets subject to 20% tax"),
    (
        "EXPLAIN_HSA_DISTRIBUTION_CENTS",
        "Actual classified cents from this owner's accepted statements and completed corrections. The detailed Form 8889 arithmetic retains cents. Pledged assets enter Schedule 1 directly. These are on-screen calculation rows, not filing-ready rounded forms; the separate estimate-transfer rows show the rounded amounts used in this estimate.",
    ),
    (
        "EXPLAIN_HSA_DISTRIBUTION_EXCEPTION",
        "One means at least one actual taxable distribution qualifies for the age, disability or death exception; zero means none. The 20% base retains actual dated payment classes, not a residual reconstructed from rounded amounts.",
    ),
    (
        "EXPLAIN_HSA_DISTRIBUTION_ESTIMATE",
        "This owner's existing estimate transfer rounds taxable distribution income to whole dollars and calculates rounded 20% tax from the rounded actual penalty base. It includes direct pledged-asset amounts. Separate spouses are rounded independently; detailed cents remain visible above.",
    ),
    ("LINE_HSA_N_DEEMED", "HSA {0} deemed"),
    ("LINE_HSA_N_EXCESS_CONTRIBUTION_TAX", "HSA {0} excess contribution tax"),
    ("LINE_HSA_N_LAST_MONTH_CONTRIBUTIONS", "HSA {0} last month contributions"),
    ("LINE_HSA_N_LATE_EXCESS_WITHDRAWALS", "HSA {0} late excess withdrawals"),
    ("LINE_HSA_N_LATER_EMPLOYER_EXCESS_INCOME", "HSA {0} later employer excess income"),
    (
        "LINE_HSA_N_PRIOR_DESIGNATED_EMPLOYER_EXCESS_INCOME",
        "HSA {0} prior designated employer excess income",
    ),
    ("LINE_HSA_N_PRIOR_EXCESS_DEDUCTED_N", "HSA {0} prior excess deducted {1}"),
    ("LINE_HSA_N_PRIOR_EXCESS_REMAINING_N", "HSA {0} prior excess remaining {1}"),
    ("LINE_HSA_N_PRIOR_EXCESS_USED_BY_DISTRIBUTIONS", "HSA {0} prior excess used by distributions"),
    ("LINE_HSA_N_PRIOR_MONTHLY_LIMIT", "HSA {0} prior monthly limit"),
    ("LINE_HSA_N_TESTING_FUNDING_INCOME", "HSA owner {0}: IRA funding testing-period income"),
    ("LINE_HSA_N_TESTING_LAST_MONTH_INCOME", "HSA owner {0}: last-month testing-period income"),
    ("LINE_HSA_N_TESTING_TAX", "HSA owner {0}: additional testing-period tax"),
    ("LINE_HSA_N_TESTING_TOTAL_INCOME", "HSA owner {0}: combined testing-period income"),
    ("LINE_HSA_N_CONTRIBUTION_PERSONAL", "HSA owner {0}: personal contributions"),
    ("LINE_HSA_N_CONTRIBUTION_LIMIT", "HSA owner {0}: contribution limit before archer reduction"),
    ("LINE_HSA_N_CONTRIBUTION_ARCHER", "HSA owner {0}: archer msa reduction"),
    ("LINE_HSA_N_CONTRIBUTION_AFTER_ARCHER", "HSA owner {0}: limit after archer reduction"),
    ("LINE_HSA_N_CONTRIBUTION_ALLOCATED", "HSA owner {0}: allocated contribution limit"),
    ("LINE_HSA_N_CONTRIBUTION_CATCHUP", "HSA owner {0}: separate married-family catch-up"),
    ("LINE_HSA_N_CONTRIBUTION_TOTAL_LIMIT", "HSA owner {0}: total owner contribution limit"),
    ("LINE_HSA_N_CONTRIBUTION_EMPLOYER", "HSA owner {0}: employer contributions"),
    ("LINE_HSA_N_CONTRIBUTION_FUNDING", "HSA owner {0}: qualified ira funding distribution"),
    (
        "LINE_HSA_N_CONTRIBUTION_OTHER",
        "HSA owner {0}: employer contributions and qualified funding",
    ),
    (
        "LINE_HSA_N_CONTRIBUTION_PERSONAL_LIMIT",
        "HSA owner {0}: personal contribution deduction limit",
    ),
    ("LINE_HSA_N_CONTRIBUTION_DEDUCTION", "HSA owner {0}: allowed personal contribution deduction"),
    (
        "EXPLAIN_HSA_CONTRIBUTION_CENTS",
        "This owner's Form 8889 Part I detail uses the existing contribution calculation, actual annual Archer contributions, spouse allocation and selected monthly/December method. Monthly worksheet fractions remain in twelfths of cents until each displayed amount is rounded to cents; independently rounded detail rows can differ by a cent from their sum. The actual combined owner limit and return deduction retain their existing rounding. A married-family catch-up is shown separately on line 7; other catch-ups belong in line 3. Line 13 includes eligible prior excess absorbed under Form 5329 line 43 and can exceed current line 2. These are estimate details, not a separately rounded filing-ready form.",
    ),
    (
        "EXPLAIN_HSA_TESTING_OWNER_FORM",
        "This owner's existing whole-dollar estimate amounts for Form 8889 Part III. Last-month and qualified IRA-funding recapture are combined before the single 10% tax calculation; spouses remain separate. This is distinct from ordinary distribution income and its 20% tax. The original source amounts and historical eligibility remain in their existing records.",
    ),
    ("LINE_HSA_N_ROLLOVER", "HSA {0} rollover"),
    (
        "LINE_HSA_N_UNCORRECTED_EMPLOYER_EXCESS_OTHER_INCOME",
        "HSA {0} uncorrected employer excess other income",
    ),
    ("LINE_HSA_ADDITIONAL_TAX", "HSA additional tax"),
    ("LINE_HSA_CORRECTION_EARNINGS", "HSA correction earnings"),
    (
        "LINE_HSA_CORRECTION_EMPLOYER_PRINCIPAL_OTHER_INCOME",
        "HSA correction employer principal other income",
    ),
    ("LINE_HSA_FUNDING_N_N", "HSA funding {0} {1}"),
    ("LINE_HSA_FUNDING_BASIS_N_N", "HSA funding basis {0} {1}"),
    ("LINE_HSA_FUNDING_RECAPTURE", "HSA funding recapture"),
    ("LINE_HSA_LAST_MONTH_RECAPTURE", "HSA last month recapture"),
    ("LINE_HSA_TAXABLE_DISTRIBUTIONS", "HSA taxable distributions"),
    ("LINE_HSA_TESTING_PERIOD_TAX", "HSA testing period tax"),
    ("LINE_INCOME_TAX", "Income tax"),
    ("LINE_INHERITED_ESTATE_TAX_DEDUCTION", "Inherited estate tax deduction"),
    ("LINE_INHERITED_IRA_N_BASIS_RECOVERED", "Inherited IRA {0} basis recovered"),
    ("LINE_INHERITED_IRA_N_BASIS_REMAINING", "Inherited IRA {0} basis remaining"),
    (
        "LINE_INHERITED_IRA_N_ESTATE_IRD_VALUE_REMAINING",
        "Inherited IRA {0} estate IRD value remaining (income in respect of a decedent)",
    ),
    ("LINE_INHERITED_IRA_N_ESTATE_TAX_DEDUCTION", "Inherited IRA {0} estate tax deduction"),
    ("LINE_INHERITED_IRA_N_TAXABLE", "Inherited IRA {0} taxable"),
    (
        "LINE_INHERITED_IRA_N_FORM_LINE1",
        "Current nondeductible IRA contributions and reservist repayments, inherited traditional IRA {0}, Form 8606 line 1",
    ),
    (
        "LINE_INHERITED_IRA_N_FORM_LINE2",
        "Opening traditional IRA basis, inherited traditional IRA {0}, Form 8606 line 2",
    ),
    (
        "LINE_INHERITED_IRA_N_FORM_LINE3",
        "Total traditional IRA basis, inherited traditional IRA {0}, Form 8606 line 3",
    ),
    (
        "LINE_INHERITED_IRA_N_FORM_LINE4",
        "Nondeductible IRA contributions paid in 2027, inherited traditional IRA {0}, Form 8606 line 4",
    ),
    (
        "LINE_INHERITED_IRA_N_FORM_LINE5",
        "Basis available for 2026 IRA distributions, inherited traditional IRA {0}, Form 8606 line 5",
    ),
    (
        "LINE_INHERITED_IRA_N_FORM_LINE6",
        "Adjusted year-end IRA value and outstanding rollovers, inherited traditional IRA {0}, Form 8606 line 6",
    ),
    (
        "LINE_INHERITED_IRA_N_FORM_LINE7",
        "Traditional IRA distributions excluding conversions, inherited traditional IRA {0}, Form 8606 line 7",
    ),
    (
        "LINE_INHERITED_IRA_N_FORM_LINE8",
        "Traditional IRA amount converted to Roth, inherited traditional IRA {0}, Form 8606 line 8",
    ),
    (
        "LINE_INHERITED_IRA_N_FORM_LINE9",
        "Total IRA value and distributions, inherited traditional IRA {0}, Form 8606 line 9",
    ),
    (
        "LINE_INHERITED_IRA_N_FORM_LINE10",
        "Nontaxable IRA fraction, inherited traditional IRA {0}, Form 8606 line 10",
    ),
    (
        "LINE_INHERITED_IRA_N_FORM_LINE11",
        "Nontaxable part of Roth conversions, inherited traditional IRA {0}, Form 8606 line 11",
    ),
    (
        "LINE_INHERITED_IRA_N_FORM_LINE12",
        "Nontaxable part of ordinary IRA distributions, inherited traditional IRA {0}, Form 8606 line 12",
    ),
    (
        "LINE_INHERITED_IRA_N_FORM_LINE13",
        "Total nontaxable IRA distributions, inherited traditional IRA {0}, Form 8606 line 13",
    ),
    (
        "LINE_INHERITED_IRA_N_FORM_LINE14",
        "Traditional IRA basis carried forward, inherited traditional IRA {0}, Form 8606 line 14",
    ),
    (
        "LINE_INHERITED_IRA_N_FORM_LINE15A",
        "Ordinary IRA taxable amount before relief, inherited traditional IRA {0}, Form 8606 line 15a",
    ),
    (
        "LINE_INHERITED_IRA_N_FORM_LINE15B",
        "IRA taxable amount allocated to qualified disasters, inherited traditional IRA {0}, Form 8606 line 15b",
    ),
    (
        "LINE_INHERITED_IRA_N_FORM_LINE15C",
        "Ordinary IRA taxable amount after repayments, inherited traditional IRA {0}, Form 8606 line 15c",
    ),
    (
        "LINE_INHERITED_ROTH_N_FORM_LINE19",
        "Nonqualified Roth distributions, inherited Roth IRA {0}, Form 8606 line 19",
    ),
    (
        "LINE_INHERITED_ROTH_N_FORM_LINE20",
        "Qualified first-home expenses, inherited Roth IRA {0}, Form 8606 line 20",
    ),
    (
        "LINE_INHERITED_ROTH_N_FORM_LINE21",
        "Roth distributions after first-home expenses, inherited Roth IRA {0}, Form 8606 line 21",
    ),
    (
        "LINE_INHERITED_ROTH_N_FORM_LINE22",
        "Available Roth contribution basis, inherited Roth IRA {0}, Form 8606 line 22",
    ),
    (
        "LINE_INHERITED_ROTH_N_FORM_LINE23",
        "Roth distributions after contribution basis, inherited Roth IRA {0}, Form 8606 line 23",
    ),
    (
        "LINE_INHERITED_ROTH_N_FORM_LINE24",
        "Available Roth conversion and rollover basis, inherited Roth IRA {0}, Form 8606 line 24",
    ),
    (
        "LINE_INHERITED_ROTH_N_FORM_LINE25A",
        "Roth earnings before relief, inherited Roth IRA {0}, Form 8606 line 25a",
    ),
    (
        "LINE_INHERITED_ROTH_N_FORM_LINE25B",
        "Roth earnings allocated to qualified disasters, inherited Roth IRA {0}, Form 8606 line 25b",
    ),
    (
        "LINE_INHERITED_ROTH_N_FORM_LINE25C",
        "Taxable Roth earnings after repayments, inherited Roth IRA {0}, Form 8606 line 25c",
    ),
    ("LINE_INHERITED_MEDICAL_N_MEDICAL", "Inherited medical {0} medical"),
    ("LINE_INHERITED_MEDICAL_N_TAXABLE", "Inherited medical {0} taxable"),
    ("LINE_INHERITED_MEDICAL_N_VALUE", "Inherited medical {0} value"),
    ("LINE_INHERITED_ROTH_N_CONVERSION_RECOVERED", "Inherited Roth {0} conversion recovered"),
    ("LINE_INHERITED_ROTH_N_CONVERSION_REMAINING", "Inherited Roth {0} conversion remaining"),
    ("LINE_INHERITED_ROTH_N_REGULAR_RECOVERED", "Inherited Roth {0} regular recovered"),
    ("LINE_INHERITED_ROTH_N_REGULAR_REMAINING", "Inherited Roth {0} regular remaining"),
    ("LINE_INHERITED_ROTH_N_TAXABLE", "Inherited Roth {0} taxable"),
    (
        "INSTALLMENT_AMOUNT_OVERFLOW",
        "Basis plus selling expenses exceeds the supported monetary encoding.",
    ),
    (
        "EXPLAIN_INSTALLMENT_PLEDGE",
        "Eligible net proceeds of a loan secured by this installment note are treated as principal received under section 453A(d), included once on Form 6252 line 21.",
    ),
    (
        "EXPLAIN_INSTALLMENT_RELATED_ACCELERATION",
        "The related buyer's second disposition accelerates the shared installment calculation's additional income under section 453(e), in addition to line 26; this nondepreciable property's ordinary recapture is zero.",
    ),
    ("LINE_INSTALLMENT_SALE_N_PLEDGED", "Installment sale {0} loan proceeds treated as payment"),
    (
        "LINE_INSTALLMENT_SALE_N_RESALE_REALIZED",
        "Installment sale {0} related buyer's amount realized",
    ),
    (
        "LINE_INSTALLMENT_SALE_N_RESALE_CONTRACT_PRICE",
        "Installment sale {0} resale contract-price limit",
    ),
    ("LINE_INSTALLMENT_SALE_N_LESSER", "Installment sale {0} limited amount realized"),
    (
        "LINE_INSTALLMENT_SALE_N_PAYMENTS_TO_DATE",
        "Installment sale {0} payments received or treated as received",
    ),
    ("LINE_INSTALLMENT_SALE_N_ACCELERATED", "Installment sale {0} accelerated principal"),
    ("LINE_INSTALLMENT_SALE_N_RESALE_INCOME", "Installment sale {0} additional resale income"),
    ("LINE_INSTALLMENT_SALE_N_RESALE_ORDINARY", "Installment sale {0} resale ordinary income"),
    ("LINE_INSTALLMENT_SALE_N_RESALE_GAIN", "Installment sale {0} additional resale gain"),
    ("LINE_INSTALLMENT_SALE_N_PROFIT", "Installment sale {0} profit before exclusion"),
    ("LINE_INSTALLMENT_SALE_N_EXCLUDED", "Installment sale {0} home exclusion"),
    ("LINE_INSTALLMENT_SALE_N_PERCENTAGE", "Installment sale {0} gross profit percentage"),
    (
        "EXPLAIN_INSTALLMENT_NO_HOME_EXCLUSION",
        "This investment or personal-use property sale claims no main-home exclusion.",
    ),
    ("LINE_INSTALLMENT_SALE_N_PRICE", "Installment sale {0} price"),
    ("LINE_INSTALLMENT_SALE_N_ASSUMED_DEBT", "Installment sale {0} assumed debt"),
    ("LINE_INSTALLMENT_SALE_N_NET_PRICE", "Installment sale {0} net price"),
    ("LINE_INSTALLMENT_SALE_N_COST", "Installment sale {0} cost"),
    ("LINE_INSTALLMENT_SALE_N_DEPRECIATION", "Installment sale {0} depreciation"),
    ("LINE_INSTALLMENT_SALE_N_ADJUSTED_BASIS", "Installment sale {0} adjusted basis"),
    ("LINE_INSTALLMENT_SALE_N_EXPENSES", "Installment sale {0} expenses"),
    ("LINE_INSTALLMENT_SALE_N_RECAPTURE", "Installment sale {0} recapture"),
    ("LINE_INSTALLMENT_SALE_N_TOTAL", "Installment sale {0} total"),
    ("LINE_INSTALLMENT_SALE_N_GROSS_PROFIT", "Installment sale {0} gross profit"),
    ("LINE_INSTALLMENT_SALE_N_EXCESS_DEBT", "Installment sale {0} excess debt"),
    ("LINE_INSTALLMENT_SALE_N_CONTRACT_PRICE", "Installment sale {0} contract price"),
    ("LINE_INSTALLMENT_SALE_N_DEEMED_PAYMENT", "Installment sale {0} deemed payment"),
    ("LINE_INSTALLMENT_SALE_N_PAYMENTS", "Installment sale {0} payments"),
    ("LINE_INSTALLMENT_SALE_N_TOTAL_PAYMENTS", "Installment sale {0} total payments"),
    ("LINE_INSTALLMENT_SALE_N_INCOME", "Installment sale {0} income"),
    ("LINE_INSTALLMENT_SALE_N_ORDINARY", "Installment sale {0} ordinary"),
    ("LINE_INSTALLMENT_SALE_N_GAIN", "Installment sale {0} gain"),
    ("LINE_INSTALLMENT_N_EARLIER_PAYMENTS", "Installment {0} earlier payments"),
    ("LINE_INSTALLMENT_N_GAIN", "Installment {0} gain"),
    ("LINE_INSTALLMENT_N_INCOME", "Installment {0} income"),
    ("LINE_INSTALLMENT_N_ORDINARY", "Installment {0} ordinary"),
    ("LINE_INSTALLMENT_N_PAYMENTS", "Installment {0} payments"),
    ("LINE_INSTALLMENT_N_SECTION1231", "Installment {0} section 1231 gain"),
    ("LINE_INSTALLMENT_N_TOTAL_PAYMENTS", "Installment {0} total payments"),
    ("LINE_INSTALLMENT_N_UNRECAPTURED_N", "Installment {0} unrecaptured {1}"),
    ("LINE_INSURANCE_EXCEPTION_N", "Insurance exception {0}"),
    ("LINE_INVESTMENT_ELECTED_CAPITAL_GAIN", "Investment elected capital gain"),
    ("LINE_INVESTMENT_ELECTED_QUALIFIED_DIVIDENDS", "Investment elected qualified dividends"),
    ("LINE_INVESTMENT_INTEREST_CARRYFORWARD", "Investment interest carryforward"),
    ("LINE_INVESTMENT_INTEREST_DEDUCTION", "Investment interest deduction"),
    ("LINE_INVESTMENT_INTEREST_NET_CAPITAL_GAIN", "Investment interest net capital gain"),
    ("LINE_INVESTMENT_INTEREST_NET_GAIN", "Investment interest net gain"),
    ("LINE_INVESTMENT_INTEREST_ORDINARY_INCOME", "Investment interest ordinary income"),
    ("LINE_INVESTMENT_PROPERTY_TAX_DEDUCTION", "Investment property tax deduction"),
    (
        "LINE_TAXPAYER_ROTH_FORM_LINE19",
        "Nonqualified Roth distributions, taxpayer, Form 8606 line 19",
    ),
    (
        "LINE_TAXPAYER_ROTH_FORM_LINE20",
        "Qualified first-home expenses, taxpayer, Form 8606 line 20",
    ),
    (
        "LINE_TAXPAYER_ROTH_FORM_LINE21",
        "Roth distributions after first-home expenses, taxpayer, Form 8606 line 21",
    ),
    (
        "LINE_TAXPAYER_ROTH_FORM_LINE22",
        "Available Roth contribution basis, taxpayer, Form 8606 line 22",
    ),
    (
        "LINE_TAXPAYER_ROTH_FORM_LINE23",
        "Roth distributions after contribution basis, taxpayer, Form 8606 line 23",
    ),
    (
        "LINE_TAXPAYER_ROTH_FORM_LINE24",
        "Available Roth conversion and rollover basis, taxpayer, Form 8606 line 24",
    ),
    (
        "LINE_TAXPAYER_ROTH_FORM_LINE25A",
        "Roth earnings before relief, taxpayer, Form 8606 line 25a",
    ),
    (
        "LINE_TAXPAYER_ROTH_FORM_LINE25B",
        "Roth earnings allocated to qualified disasters, taxpayer, Form 8606 line 25b",
    ),
    (
        "LINE_TAXPAYER_ROTH_FORM_LINE25C",
        "Taxable Roth earnings after repayments, taxpayer, Form 8606 line 25c",
    ),
    ("LINE_SPOUSE_ROTH_FORM_LINE19", "Nonqualified Roth distributions, spouse, Form 8606 line 19"),
    ("LINE_SPOUSE_ROTH_FORM_LINE20", "Qualified first-home expenses, spouse, Form 8606 line 20"),
    (
        "LINE_SPOUSE_ROTH_FORM_LINE21",
        "Roth distributions after first-home expenses, spouse, Form 8606 line 21",
    ),
    (
        "LINE_SPOUSE_ROTH_FORM_LINE22",
        "Available Roth contribution basis, spouse, Form 8606 line 22",
    ),
    (
        "LINE_SPOUSE_ROTH_FORM_LINE23",
        "Roth distributions after contribution basis, spouse, Form 8606 line 23",
    ),
    (
        "LINE_SPOUSE_ROTH_FORM_LINE24",
        "Available Roth conversion and rollover basis, spouse, Form 8606 line 24",
    ),
    ("LINE_SPOUSE_ROTH_FORM_LINE25A", "Roth earnings before relief, spouse, Form 8606 line 25a"),
    (
        "LINE_SPOUSE_ROTH_FORM_LINE25B",
        "Roth earnings allocated to qualified disasters, spouse, Form 8606 line 25b",
    ),
    (
        "LINE_SPOUSE_ROTH_FORM_LINE25C",
        "Taxable Roth earnings after repayments, spouse, Form 8606 line 25c",
    ),
    (
        "LINE_TAXPAYER_IRA_FORM_LINE1",
        "Current nondeductible IRA contributions and reservist repayments, taxpayer, Form 8606 line 1",
    ),
    ("LINE_TAXPAYER_IRA_FORM_LINE2", "Opening traditional IRA basis, taxpayer, Form 8606 line 2"),
    ("LINE_TAXPAYER_IRA_FORM_LINE3", "Total traditional IRA basis, taxpayer, Form 8606 line 3"),
    (
        "LINE_TAXPAYER_IRA_FORM_LINE4",
        "Nondeductible IRA contributions paid in 2027, taxpayer, Form 8606 line 4",
    ),
    (
        "LINE_TAXPAYER_IRA_FORM_LINE5",
        "Basis available for 2026 IRA distributions, taxpayer, Form 8606 line 5",
    ),
    (
        "LINE_TAXPAYER_IRA_FORM_LINE6",
        "Adjusted year-end IRA value and outstanding rollovers, taxpayer, Form 8606 line 6",
    ),
    (
        "LINE_TAXPAYER_IRA_FORM_LINE7",
        "Traditional IRA distributions excluding conversions, taxpayer, Form 8606 line 7",
    ),
    (
        "LINE_TAXPAYER_IRA_FORM_LINE8",
        "Traditional IRA amount converted to Roth, taxpayer, Form 8606 line 8",
    ),
    (
        "LINE_TAXPAYER_IRA_FORM_LINE9",
        "Total IRA value and distributions, taxpayer, Form 8606 line 9",
    ),
    ("LINE_TAXPAYER_IRA_FORM_LINE10", "Nontaxable IRA fraction, taxpayer, Form 8606 line 10"),
    (
        "LINE_TAXPAYER_IRA_FORM_LINE11",
        "Nontaxable part of Roth conversions, taxpayer, Form 8606 line 11",
    ),
    (
        "LINE_TAXPAYER_IRA_FORM_LINE12",
        "Nontaxable part of ordinary IRA distributions, taxpayer, Form 8606 line 12",
    ),
    (
        "LINE_TAXPAYER_IRA_FORM_LINE13",
        "Total nontaxable IRA distributions, taxpayer, Form 8606 line 13",
    ),
    (
        "LINE_TAXPAYER_IRA_FORM_LINE14",
        "Traditional IRA basis carried forward, taxpayer, Form 8606 line 14",
    ),
    (
        "LINE_TAXPAYER_IRA_FORM_LINE15A",
        "Ordinary IRA taxable amount before relief, taxpayer, Form 8606 line 15a",
    ),
    (
        "LINE_TAXPAYER_IRA_FORM_LINE15B",
        "IRA taxable amount allocated to qualified disasters, taxpayer, Form 8606 line 15b",
    ),
    (
        "LINE_TAXPAYER_IRA_FORM_LINE15C",
        "Ordinary IRA taxable amount after repayments, taxpayer, Form 8606 line 15c",
    ),
    (
        "LINE_TAXPAYER_IRA_FORM_LINE16",
        "Traditional IRA amount converted to Roth, taxpayer, Form 8606 line 16",
    ),
    (
        "LINE_TAXPAYER_IRA_FORM_LINE17",
        "Basis allocated to Roth conversions, taxpayer, Form 8606 line 17",
    ),
    (
        "LINE_TAXPAYER_IRA_FORM_LINE18",
        "Conversion balance taxable only if positive, taxpayer, Form 8606 line 18",
    ),
    (
        "LINE_SPOUSE_IRA_FORM_LINE1",
        "Current nondeductible IRA contributions and reservist repayments, spouse, Form 8606 line 1",
    ),
    ("LINE_SPOUSE_IRA_FORM_LINE2", "Opening traditional IRA basis, spouse, Form 8606 line 2"),
    ("LINE_SPOUSE_IRA_FORM_LINE3", "Total traditional IRA basis, spouse, Form 8606 line 3"),
    (
        "LINE_SPOUSE_IRA_FORM_LINE4",
        "Nondeductible IRA contributions paid in 2027, spouse, Form 8606 line 4",
    ),
    (
        "LINE_SPOUSE_IRA_FORM_LINE5",
        "Basis available for 2026 IRA distributions, spouse, Form 8606 line 5",
    ),
    (
        "LINE_SPOUSE_IRA_FORM_LINE6",
        "Adjusted year-end IRA value and outstanding rollovers, spouse, Form 8606 line 6",
    ),
    (
        "LINE_SPOUSE_IRA_FORM_LINE7",
        "Traditional IRA distributions excluding conversions, spouse, Form 8606 line 7",
    ),
    (
        "LINE_SPOUSE_IRA_FORM_LINE8",
        "Traditional IRA amount converted to Roth, spouse, Form 8606 line 8",
    ),
    ("LINE_SPOUSE_IRA_FORM_LINE9", "Total IRA value and distributions, spouse, Form 8606 line 9"),
    ("LINE_SPOUSE_IRA_FORM_LINE10", "Nontaxable IRA fraction, spouse, Form 8606 line 10"),
    (
        "LINE_SPOUSE_IRA_FORM_LINE11",
        "Nontaxable part of Roth conversions, spouse, Form 8606 line 11",
    ),
    (
        "LINE_SPOUSE_IRA_FORM_LINE12",
        "Nontaxable part of ordinary IRA distributions, spouse, Form 8606 line 12",
    ),
    (
        "LINE_SPOUSE_IRA_FORM_LINE13",
        "Total nontaxable IRA distributions, spouse, Form 8606 line 13",
    ),
    (
        "LINE_SPOUSE_IRA_FORM_LINE14",
        "Traditional IRA basis carried forward, spouse, Form 8606 line 14",
    ),
    (
        "LINE_SPOUSE_IRA_FORM_LINE15A",
        "Ordinary IRA taxable amount before relief, spouse, Form 8606 line 15a",
    ),
    (
        "LINE_SPOUSE_IRA_FORM_LINE15B",
        "IRA taxable amount allocated to qualified disasters, spouse, Form 8606 line 15b",
    ),
    (
        "LINE_SPOUSE_IRA_FORM_LINE15C",
        "Ordinary IRA taxable amount after repayments, spouse, Form 8606 line 15c",
    ),
    (
        "LINE_SPOUSE_IRA_FORM_LINE16",
        "Traditional IRA amount converted to Roth, spouse, Form 8606 line 16",
    ),
    (
        "LINE_SPOUSE_IRA_FORM_LINE17",
        "Basis allocated to Roth conversions, spouse, Form 8606 line 17",
    ),
    (
        "LINE_SPOUSE_IRA_FORM_LINE18",
        "Conversion balance taxable only if positive, spouse, Form 8606 line 18",
    ),
    ("LINE_IRA_DEDUCTION", "IRA deduction"),
    (
        "LINE_TAXPAYER_RMD_LINE52A",
        "Minimum distributions for plans corrected in time, taxpayer, Form 5329 line 52a",
    ),
    (
        "LINE_TAXPAYER_RMD_LINE52B",
        "Minimum distributions for other plans, taxpayer, Form 5329 line 52b",
    ),
    (
        "LINE_TAXPAYER_RMD_LINE53A",
        "Timely distributions for plans corrected in time, taxpayer, Form 5329 line 53a",
    ),
    (
        "LINE_TAXPAYER_RMD_LINE53B",
        "Timely distributions for other plans, taxpayer, Form 5329 line 53b",
    ),
    ("LINE_TAXPAYER_RMD_LINE54A", "Additional tax at 10%, taxpayer, Form 5329 line 54a"),
    ("LINE_TAXPAYER_RMD_LINE54B", "Additional tax at 25%, taxpayer, Form 5329 line 54b"),
    (
        "LINE_TAXPAYER_RMD_LINE55",
        "Required-distribution additional tax, taxpayer, Form 5329 line 55",
    ),
    ("LINE_TAXPAYER_RMD_WAIVER54A", "Waiver requested, taxpayer, Form 5329 line 54a"),
    ("LINE_TAXPAYER_RMD_WAIVER54B", "Waiver requested, taxpayer, Form 5329 line 54b"),
    (
        "LINE_SPOUSE_RMD_LINE52A",
        "Minimum distributions for plans corrected in time, spouse, Form 5329 line 52a",
    ),
    (
        "LINE_SPOUSE_RMD_LINE52B",
        "Minimum distributions for other plans, spouse, Form 5329 line 52b",
    ),
    (
        "LINE_SPOUSE_RMD_LINE53A",
        "Timely distributions for plans corrected in time, spouse, Form 5329 line 53a",
    ),
    ("LINE_SPOUSE_RMD_LINE53B", "Timely distributions for other plans, spouse, Form 5329 line 53b"),
    ("LINE_SPOUSE_RMD_LINE54A", "Additional tax at 10%, spouse, Form 5329 line 54a"),
    ("LINE_SPOUSE_RMD_LINE54B", "Additional tax at 25%, spouse, Form 5329 line 54b"),
    ("LINE_SPOUSE_RMD_LINE55", "Required-distribution additional tax, spouse, Form 5329 line 55"),
    ("LINE_SPOUSE_RMD_WAIVER54A", "Waiver requested, spouse, Form 5329 line 54a"),
    ("LINE_SPOUSE_RMD_WAIVER54B", "Waiver requested, spouse, Form 5329 line 54b"),
    (
        "EXPLAIN_RMD_FORM_COMPARISON",
        "Each legally separate plan keeps its own shortfall. Only timely distributions count, capped by that plan or permitted group's minimum. A full correction within the correction window uses 10%; other shortfalls use 25%. Each filer's rates are rounded separately.",
    ),
    (
        "EXPLAIN_RMD_FORM_WAIVER_REQUEST",
        "This is the shortfall requested to be waived for reasonable error and remedial steps, shown as the RC supporting amount. The estimate subtracts the request before calculating tax; IRS approval is still pending.",
    ),
    ("LINE_HSA_N_EXCESS_LINE42", "HSA excess, filer {0}, Form 5329 line 42"),
    ("LINE_HSA_N_EXCESS_LINE43", "HSA excess, filer {0}, Form 5329 line 43"),
    ("LINE_HSA_N_EXCESS_LINE44", "HSA excess, filer {0}, Form 5329 line 44"),
    ("LINE_HSA_N_EXCESS_LINE45", "HSA excess, filer {0}, Form 5329 line 45"),
    ("LINE_HSA_N_EXCESS_LINE46", "HSA excess, filer {0}, Form 5329 line 46"),
    ("LINE_HSA_N_EXCESS_LINE47", "HSA excess, filer {0}, Form 5329 line 47"),
    ("LINE_HSA_N_EXCESS_LINE48", "HSA excess, filer {0}, Form 5329 line 48"),
    ("LINE_HSA_N_EXCESS_LINE49", "HSA excess, filer {0}, Form 5329 line 49"),
    ("LINE_ARCHER_N_EXCESS_LINE34", "Archer MSA excess, filer {0}, Form 5329 line 34"),
    ("LINE_ARCHER_N_EXCESS_LINE35", "Archer MSA excess, filer {0}, Form 5329 line 35"),
    ("LINE_ARCHER_N_EXCESS_LINE36", "Archer MSA excess, filer {0}, Form 5329 line 36"),
    ("LINE_ARCHER_N_EXCESS_LINE37", "Archer MSA excess, filer {0}, Form 5329 line 37"),
    ("LINE_ARCHER_N_EXCESS_LINE38", "Archer MSA excess, filer {0}, Form 5329 line 38"),
    ("LINE_ARCHER_N_EXCESS_LINE39", "Archer MSA excess, filer {0}, Form 5329 line 39"),
    ("LINE_ARCHER_N_EXCESS_LINE40", "Archer MSA excess, filer {0}, Form 5329 line 40"),
    ("LINE_ARCHER_N_EXCESS_LINE41", "Archer MSA excess, filer {0}, Form 5329 line 41"),
    (
        "EXPLAIN_HEALTH_ACCOUNT_EXCESS_FORM",
        "The prior excess is reduced by taxable distributions and unused contribution room. The remaining prior excess and current excess are combined, and the additional tax is limited by the retained account value. Contribution and account amounts include cents; additional tax is rounded to whole dollars.",
    ),
    (
        "LINE_IRA_EXCESS_SPOUSE_LINE10_UNUSED_TRADITIONAL_ROOM",
        "IRA excess line 10 unused traditional room, spouse",
    ),
    (
        "LINE_IRA_EXCESS_SPOUSE_LINE11_TRADITIONAL_DISTRIBUTIONS",
        "IRA excess line 11 traditional distributions, spouse",
    ),
    (
        "LINE_IRA_EXCESS_SPOUSE_LINE14_PRIOR_TRADITIONAL_REMAINING",
        "IRA excess line 14 prior traditional remaining, spouse",
    ),
    (
        "LINE_IRA_EXCESS_SPOUSE_LINE15_TRADITIONAL_EXCESS",
        "IRA excess line 15 traditional excess, spouse",
    ),
    (
        "LINE_IRA_EXCESS_SPOUSE_LINE16_TRADITIONAL_TOTAL_EXCESS",
        "IRA excess line 16 traditional total excess, spouse",
    ),
    (
        "LINE_IRA_EXCESS_SPOUSE_LINE17_TRADITIONAL_EXCESS_TAX",
        "IRA excess line 17 traditional excess tax, spouse",
    ),
    (
        "LINE_IRA_EXCESS_SPOUSE_LINE18_PRIOR_ROTH_EXCESS",
        "IRA excess line 18 prior Roth excess, spouse",
    ),
    (
        "LINE_IRA_EXCESS_SPOUSE_LINE19_UNUSED_ROTH_ROOM",
        "IRA excess line 19 unused Roth room, spouse",
    ),
    (
        "LINE_IRA_EXCESS_SPOUSE_LINE20_ROTH_DISTRIBUTIONS",
        "IRA excess line 20 Roth distributions, spouse",
    ),
    (
        "LINE_IRA_EXCESS_SPOUSE_LINE22_PRIOR_ROTH_REMAINING",
        "IRA excess line 22 prior Roth remaining, spouse",
    ),
    ("LINE_IRA_EXCESS_SPOUSE_LINE23_ROTH_EXCESS", "IRA excess line 23 Roth excess, spouse"),
    (
        "LINE_IRA_EXCESS_SPOUSE_LINE24_ROTH_TOTAL_EXCESS",
        "IRA excess line 24 Roth total excess, spouse",
    ),
    ("LINE_IRA_EXCESS_SPOUSE_LINE25_ROTH_EXCESS_TAX", "IRA excess line 25 Roth excess tax, spouse"),
    (
        "LINE_IRA_EXCESS_SPOUSE_LINE9_PRIOR_TRADITIONAL_EXCESS",
        "IRA excess line 9 prior traditional excess, spouse",
    ),
    ("LINE_TRUMP_ACCOUNT_SPOUSE_LINE57_EXCESS", "Trump account line 57 excess, spouse"),
    ("LINE_TRUMP_ACCOUNT_SPOUSE_LINE59_RETURNED", "Trump account line 59 returned, spouse"),
    ("LINE_TRUMP_ACCOUNT_SPOUSE_LINE60_REMAINING", "Trump account line 60 remaining, spouse"),
    ("LINE_TRUMP_ACCOUNT_SPOUSE_LINE61_TAX", "Trump account line 61 tax, spouse"),
    ("LINE_TRUMP_ACCOUNT_SPOUSE_LINE62_EARNINGS", "Trump account line 62 earnings, spouse"),
    ("LINE_TRUMP_ACCOUNT_SPOUSE_LINE63_TAX", "Trump account line 63 tax, spouse"),
    (
        "LINE_IRA_EXCESS_TAXPAYER_LINE13_TRADITIONAL_TOTAL_REDUCTION",
        "Traditional IRA excess reduction, taxpayer, Form 5329 line 13",
    ),
    (
        "LINE_IRA_EXCESS_TAXPAYER_LINE21_ROTH_TOTAL_REDUCTION",
        "Roth IRA excess reduction, taxpayer, Form 5329 line 21",
    ),
    (
        "LINE_IRA_EXCESS_SPOUSE_LINE13_TRADITIONAL_TOTAL_REDUCTION",
        "Traditional IRA excess reduction, spouse, Form 5329 line 13",
    ),
    (
        "LINE_IRA_EXCESS_SPOUSE_LINE21_ROTH_TOTAL_REDUCTION",
        "Roth IRA excess reduction, spouse, Form 5329 line 21",
    ),
    (
        "LINE_IRA_EXCESS_TAXPAYER_LINE10_UNUSED_TRADITIONAL_ROOM",
        "IRA excess line 10 unused traditional room, taxpayer",
    ),
    (
        "LINE_IRA_EXCESS_TAXPAYER_LINE11_TRADITIONAL_DISTRIBUTIONS",
        "IRA excess line 11 traditional distributions, taxpayer",
    ),
    (
        "LINE_IRA_EXCESS_TAXPAYER_LINE14_PRIOR_TRADITIONAL_REMAINING",
        "IRA excess line 14 prior traditional remaining, taxpayer",
    ),
    (
        "LINE_IRA_EXCESS_TAXPAYER_LINE15_TRADITIONAL_EXCESS",
        "IRA excess line 15 traditional excess, taxpayer",
    ),
    (
        "LINE_IRA_EXCESS_TAXPAYER_LINE16_TRADITIONAL_TOTAL_EXCESS",
        "IRA excess line 16 traditional total excess, taxpayer",
    ),
    (
        "LINE_IRA_EXCESS_TAXPAYER_LINE17_TRADITIONAL_EXCESS_TAX",
        "IRA excess line 17 traditional excess tax, taxpayer",
    ),
    (
        "LINE_IRA_EXCESS_TAXPAYER_LINE18_PRIOR_ROTH_EXCESS",
        "IRA excess line 18 prior Roth excess, taxpayer",
    ),
    (
        "LINE_IRA_EXCESS_TAXPAYER_LINE19_UNUSED_ROTH_ROOM",
        "IRA excess line 19 unused Roth room, taxpayer",
    ),
    (
        "LINE_IRA_EXCESS_TAXPAYER_LINE20_ROTH_DISTRIBUTIONS",
        "IRA excess line 20 Roth distributions, taxpayer",
    ),
    (
        "LINE_IRA_EXCESS_TAXPAYER_LINE22_PRIOR_ROTH_REMAINING",
        "IRA excess line 22 prior Roth remaining, taxpayer",
    ),
    ("LINE_IRA_EXCESS_TAXPAYER_LINE23_ROTH_EXCESS", "IRA excess line 23 Roth excess, taxpayer"),
    (
        "LINE_IRA_EXCESS_TAXPAYER_LINE24_ROTH_TOTAL_EXCESS",
        "IRA excess line 24 Roth total excess, taxpayer",
    ),
    (
        "LINE_IRA_EXCESS_TAXPAYER_LINE25_ROTH_EXCESS_TAX",
        "IRA excess line 25 Roth excess tax, taxpayer",
    ),
    (
        "LINE_IRA_EXCESS_TAXPAYER_LINE9_PRIOR_TRADITIONAL_EXCESS",
        "IRA excess line 9 prior traditional excess, taxpayer",
    ),
    ("LINE_ISO_SALE_N_AMT_BASIS", "ISO sale {0} AMT basis"),
    ("LINE_ISO_SALE_N_AMT_CAPITAL_GAIN", "ISO sale {0} AMT capital gain"),
    ("LINE_ISO_SALE_N_AMT_COMPENSATION_ADJUSTMENT", "ISO sale {0} AMT compensation adjustment"),
    ("LINE_ISO_SALE_N_CAPITAL_GAIN", "ISO sale {0} capital gain"),
    ("LINE_ISO_SALE_N_COMPENSATION", "ISO sale {0} compensation"),
    ("LINE_ISO_SALE_N_REGULAR_BASIS", "ISO sale {0} regular basis"),
    ("LINE_ISO_UNREPORTED_COMPENSATION", "ISO unreported compensation"),
    ("LINE_ITEMIZED_DEDUCTION_REDUCTION", "Itemized deduction reduction"),
    ("LINE_EMPLOYEE_WAGES_N_PAY", "Employee wages {0} pay"),
    ("LINE_EMPLOYEE_WAGES_N_WAGE_BASE", "Employee wages {0} annual Social Security wage base"),
    (
        "LINE_EMPLOYEE_WAGES_N_USED_BASE",
        "Employee wages {0} Social Security wage base already used",
    ),
    (
        "LINE_EMPLOYEE_WAGES_N_REMAINING_BASE",
        "Employee wages {0} remaining Social Security wage base",
    ),
    ("LINE_EMPLOYEE_WAGES_N_SOCIAL_BASE", "Employee wages {0} Social Security base"),
    ("LINE_EMPLOYEE_WAGES_N_TAX", "Employee wages {0} Form 8919 tax"),
    ("LINE_JURY_PAY", "Jury pay"),
    ("LINE_JURY_PAY_REMITTED", "Jury pay remitted"),
    ("LINE_KIDDIE_TAX_ALLOCATED_N_PERCENT_GAIN", "Kiddie tax allocated {0} percent gain"),
    ("LINE_KIDDIE_TAX_ALLOCATED_CAPITAL_GAIN", "Kiddie tax allocated capital gain"),
    ("LINE_KIDDIE_TAX_ALLOCATED_DIVIDENDS", "Kiddie tax allocated dividends"),
    ("LINE_KIDDIE_TAX_DIRECT_GAMBLING_DEDUCTION", "Kiddie tax direct gambling deduction"),
    ("LINE_KIDDIE_TAX_NET_UNEARNED_INCOME", "Kiddie tax net unearned income"),
    ("LINE_KIDDIE_TAX_ORIGINAL_N_PERCENT_GAIN", "Kiddie tax original {0} percent gain"),
    ("LINE_KIDDIE_TAX_ORIGINAL_NET_CAPITAL_GAIN", "Kiddie tax original net capital gain"),
    ("LINE_KIDDIE_TAX_PARENT_SHARE", "Kiddie tax parent share"),
    ("LINE_KIDDIE_TAX_PREFERENTIAL_INTEREST", "Kiddie tax preferential interest"),
    ("LINE_KIDDIE_TAX_UNEARNED_INCOME", "Kiddie tax unearned income"),
    ("LINE_LEGACY_ANNUITY_N_NEW_BASIS_RECOVERY", "Legacy annuity {0} new basis recovery"),
    ("LINE_LEGACY_ANNUITY_N_NEW_BASIS_REMAINING", "Legacy annuity {0} new basis remaining"),
    ("LINE_LEGACY_ANNUITY_N_NEW_TAXABLE_EARNINGS", "Legacy annuity {0} new taxable earnings"),
    ("LINE_LEGACY_ANNUITY_N_OLD_BASIS_RECOVERY", "Legacy annuity {0} old basis recovery"),
    ("LINE_LEGACY_ANNUITY_N_OLD_BASIS_REMAINING", "Legacy annuity {0} old basis remaining"),
    ("LINE_LEGACY_ANNUITY_N_OLD_TAXABLE_EARNINGS", "Legacy annuity {0} old taxable earnings"),
    ("LINE_LEGACY_EV_PASSIVE_CREDIT", "Legacy EV passive credit"),
    ("LINE_LUMP_SUM_N_LINE_N", "Form 4972 for 1099-R {0} line {1}"),
    ("LINE_LUMP_SUM_TAX", "Tax on lump-sum distributions"),
    ("LINE_LEGACY_EV_PASSIVE_CREDIT_UNUSABLE", "Legacy EV passive credit unusable"),
    ("LINE_LONG_TERM_CARE_N_ELIGIBLE", "Long-term care {0} eligible"),
    ("LINE_LONG_TERM_CARE_N_EXCLUDED", "Long-term care {0} excluded"),
    ("LINE_LONG_TERM_CARE_N_MEDICAL", "Long-term care {0} medical"),
    ("LINE_LTC_BENEFIT_N_N_EXCLUDED", "LTC benefit {0} {1} excluded"),
    ("LINE_LTC_BENEFIT_N_N_ALLOCATED_LIMIT", "LTC benefit {0} {1} allocated limit"),
    ("LINE_LTC_BENEFIT_N_N_LIMIT", "LTC benefit {0} {1} limit"),
    ("LINE_LTC_BENEFIT_N_N_RECEIVED", "LTC benefit {0} {1} received"),
    ("LINE_LTC_BENEFIT_N_N_TAXABLE", "LTC benefit {0} {1} taxable"),
    ("LINE_LTC_BENEFIT_N_EXEMPT", "LTC benefit {0} exempt"),
    ("LINE_MARKETPLACE_ANNUAL_CONTRIBUTION", "Marketplace annual contribution"),
    ("LINE_MARKETPLACE_HOUSEHOLD_INCOME", "Marketplace household income"),
    ("LINE_MARKETPLACE_POVERTY_LINE", "Marketplace poverty line"),
    ("LINE_MARKETPLACE_SPECIFIED_DEDUCTION", "Marketplace specified worksheet component"),
    ("LINE_MARKETPLACE_SPECIFIED_MAXIMUM", "Marketplace specified maximum"),
    ("LINE_MARKETPLACE_SPECIFIED_PREMIUMS", "Marketplace specified premiums"),
    ("LINE_MEDICARE_MSA_N_ADDITIONAL_TAX", "Medicare MSA {0} additional tax"),
    ("LINE_MEDICARE_MSA_N_BALANCE_RELIEF", "Medicare MSA {0} balance relief"),
    ("LINE_MEDICARE_MSA_N_DEEMED", "Medicare MSA {0} deemed"),
    ("LINE_MEDICARE_MSA_N_EXCEPTION", "Medicare MSA {0} exception"),
    ("LINE_MEDICARE_MSA_N_TAXABLE", "Medicare MSA {0} taxable"),
    ("LINE_MEDICARE_MSA_ADDITIONAL_TAX", "Medicare MSA additional tax"),
    ("LINE_MEDICARE_MSA_TAXABLE_DISTRIBUTIONS", "Medicare MSA taxable distributions"),
    ("LINE_MINIMUM_TAX_CREDIT", "Minimum tax credit"),
    ("LINE_MINIMUM_TAX_CREDIT_AVAILABLE", "Minimum tax credit available"),
    ("LINE_MINIMUM_TAX_CREDIT_CARRYFORWARD", "Minimum tax credit carryforward"),
    ("LINE_MINIMUM_TAX_CREDIT_GENERATED", "Minimum tax credit generated"),
    ("LINE_MINIMUM_TAX_CREDIT_LIMIT", "Minimum tax credit limit"),
    ("LINE_MINIMUM_TAX_EXCLUSION_TAX", "Minimum tax exclusion tax"),
    ("LINE_MORTGAGE_AVERAGE_BALANCE", "Mortgage average balance"),
    ("LINE_MORTGAGE_CERTIFIED_INTEREST", "Mortgage certified interest"),
    ("LINE_MORTGAGE_CREDIT_CARRY_N_TO_N", "Mortgage credit carry {0} to {1}"),
    ("LINE_MORTGAGE_CREDIT_CURRENT_POTENTIAL", "Mortgage credit current potential"),
    ("LINE_MORTGAGE_CREDIT_EXPIRED_N", "Mortgage credit expired {0}"),
    ("LINE_MORTGAGE_INSURANCE_PREMIUMS_8D", "Mortgage insurance premiums 8d"),
    ("LINE_MORTGAGE_INTEREST_8A", "Mortgage interest 8a"),
    ("LINE_MORTGAGE_INTEREST_8B", "Mortgage interest 8b"),
    ("LINE_MORTGAGE_INTEREST_8E", "Mortgage interest 8e"),
    ("LINE_MORTGAGE_INTEREST_CREDIT", "Mortgage interest credit"),
    ("LINE_MORTGAGE_NONDEDUCTIBLE_INTEREST", "Mortgage nondeductible interest"),
    ("LINE_MORTGAGE_ORIGINAL_PERIOD_CREDIT", "Mortgage original period credit"),
    ("LINE_MORTGAGE_ORIGINAL_SCHEDULE_CAP", "Mortgage original schedule cap"),
    ("LINE_MORTGAGE_POINTS_8C", "Mortgage points 8c"),
    ("LINE_MORTGAGE_QUALIFIED_LOAN_LIMIT", "Mortgage qualified loan limit"),
    ("LINE_MORTGAGE_REISSUED_PERIOD_CREDIT", "Mortgage reissued period credit"),
    (
        "LINE_MORTGAGE_SUBSIDY_N_ADJUSTED_QUALIFYING_INCOME",
        "Mortgage subsidy {0} adjusted qualifying income",
    ),
    ("LINE_MORTGAGE_SUBSIDY_N_AMOUNT", "Mortgage subsidy {0} amount"),
    ("LINE_MORTGAGE_SUBSIDY_N_DISPOSITION_GAIN", "Mortgage subsidy {0} disposition gain"),
    ("LINE_MORTGAGE_SUBSIDY_N_MODIFIED_INCOME", "Mortgage subsidy {0} modified income"),
    (
        "LINE_MORTGAGE_SUBSIDY_N_QSML_PAID_GAIN_SHARE",
        "Mortgage subsidy {0} QSML paid gain share (qualified subordinate mortgage loan, Form 8828)",
    ),
    ("LINE_MORTGAGE_SUBSIDY_N_RECAPTURE_TAX", "Mortgage subsidy {0} recapture tax"),
    (
        "LINE_MORTGAGE_SUBSIDY_ADJUSTED_QUALIFYING_INCOME",
        "Mortgage subsidy adjusted qualifying income",
    ),
    ("LINE_MORTGAGE_SUBSIDY_AMOUNT", "Mortgage subsidy amount"),
    ("LINE_MORTGAGE_SUBSIDY_DISPOSITION_GAIN", "Mortgage subsidy disposition gain"),
    ("LINE_MORTGAGE_SUBSIDY_MODIFIED_INCOME", "Mortgage subsidy modified income"),
    (
        "LINE_MORTGAGE_SUBSIDY_QSML_PAID_GAIN_SHARE",
        "Mortgage subsidy QSML paid gain share (qualified subordinate mortgage loan, Form 8828)",
    ),
    ("LINE_MORTGAGE_SUBSIDY_RECAPTURE_TAX", "Mortgage subsidy recapture tax"),
    ("LINE_NET_INVESTMENT_INCOME_TAX", "Net investment income tax"),
    ("LINE_NET_INVESTMENT_MODIFIED_AGI", "Net investment modified AGI"),
    ("LINE_NEW_CLEAN_VEHICLE_CREDIT", "New clean vehicle credit"),
    ("LINE_NEW_CLEAN_VEHICLE_CREDIT_UNUSABLE", "New clean vehicle credit unusable"),
    ("LINE_NEW_CLEAN_VEHICLE_TRANSFER_REPAYMENT", "New clean vehicle transfer repayment"),
    ("LINE_NIIT_CAPITAL_CARRY_ADJUSTMENT", "NIIT capital carry adjustment"),
    ("LINE_NIIT_DISPOSITION_ADJUSTMENT", "NIIT disposition adjustment"),
    ("LINE_NIIT_INCOME_TAX_DEDUCTION", "NIIT income tax deduction"),
    ("LINE_NIIT_ITEMIZED_DEDUCTION_REDUCTION", "NIIT itemized deduction reduction"),
    ("LINE_NIIT_ITEMIZED_DEDUCTIONS", "NIIT itemized deductions"),
    ("LINE_NOL_N_CARRYFORWARD", "NOL {0} carryforward"),
    ("LINE_NOL_N_DEBT_REDUCTION", "NOL {0} debt reduction"),
    ("LINE_NOL_N_EXPIRED", "NOL {0} expired"),
    ("LINE_NOL_N_OPENING", "NOL {0} opening"),
    ("LINE_NOL_N_USED", "NOL {0} used"),
    ("LINE_NOL_CARRYFORWARD", "NOL carryforward"),
    ("LINE_NONBUSINESS_PERSONAL_RENT", "Nonbusiness personal rent"),
    ("LINE_NONBUSINESS_PERSONAL_RENT_DEPRECIATION", "Nonbusiness personal rent depreciation"),
    ("LINE_NONBUSINESS_PERSONAL_RENT_EXPENSES", "Nonbusiness personal rent expenses"),
    ("LINE_NONBUSINESS_PERSONAL_RENT_EXPENSES_PAID", "Nonbusiness personal rent expenses paid"),
    ("LINE_NONBUSINESS_ROYALTY_INCOME", "Nonbusiness royalty income"),
    ("LINE_NONQUALIFIED_ANNUITY_INVESTMENT_INCOME", "Nonqualified annuity investment income"),
    ("LINE_NUA_SALE_N_BASIS", "NUA sale {0} basis (net unrealized appreciation)"),
    (
        "LINE_NUA_SALE_N_POST_DISTRIBUTION_GAIN",
        "NUA sale {0} post distribution gain (net unrealized appreciation)",
    ),
    ("LINE_NUA_SALE_N_REALIZED_NUA", "NUA sale {0} realized NUA (net unrealized appreciation)"),
    ("LINE_OPTIONAL_SE_SPOUSE_FARM", "Optional SE farm, spouse"),
    ("LINE_OPTIONAL_SE_SPOUSE_NET", "Optional SE net, spouse"),
    ("LINE_OPTIONAL_SE_SPOUSE_NONFARM", "Optional SE nonfarm, spouse"),
    ("LINE_OPTIONAL_SE_TAXPAYER_FARM", "Optional SE farm, taxpayer"),
    ("LINE_OPTIONAL_SE_TAXPAYER_NET", "Optional SE net, taxpayer"),
    ("LINE_OPTIONAL_SE_TAXPAYER_NONFARM", "Optional SE nonfarm, taxpayer"),
    ("LINE_OVERTIME_DEDUCTION", "Overtime deduction"),
    ("LINE_PARTNERSHIP_N_ACTIVITY_NET", "Partnership {0} activity net"),
    ("LINE_PARTNERSHIP_N_BACKUP_WITHHOLDING", "Partnership {0} backup withholding"),
    ("LINE_PARTNERSHIP_N_CASH_CHARITY", "Partnership {0} cash charity"),
    ("LINE_PARTNERSHIP_N_DIVIDENDS", "Partnership {0} dividends"),
    ("LINE_PARTNERSHIP_N_QUALIFIED_DIVIDENDS", "Partnership {0} qualified dividends"),
    ("LINE_AMT_PARTNERSHIP_N_DIVIDENDS", "AMT partnership {0} dividends"),
    ("LINE_AMT_PARTNERSHIP_N_QUALIFIED_DIVIDENDS", "AMT partnership {0} qualified dividends"),
    ("LINE_PARTNERSHIP_N_EXEMPT_INTEREST", "Partnership {0} exempt interest"),
    ("LINE_PARTNERSHIP_N_FUEL_11A", "Partnership {0} fuel 11a"),
    ("LINE_PARTNERSHIP_N_FUEL_11B", "Partnership {0} fuel 11b"),
    ("LINE_PARTNERSHIP_N_FUEL_11C", "Partnership {0} fuel 11c"),
    ("LINE_PARTNERSHIP_N_FUEL_11D", "Partnership {0} fuel 11d"),
    ("LINE_PARTNERSHIP_N_FUEL_11E", "Partnership {0} fuel 11e"),
    ("LINE_PARTNERSHIP_N_FUEL_11F", "Partnership {0} fuel 11f"),
    ("LINE_PARTNERSHIP_N_FUEL_11G", "Partnership {0} fuel 11g"),
    ("LINE_PARTNERSHIP_N_FUEL_11H", "Partnership {0} fuel 11h"),
    ("LINE_PARTNERSHIP_N_FUEL_13A", "Partnership {0} fuel 13a"),
    ("LINE_PARTNERSHIP_N_FUEL_13B", "Partnership {0} fuel 13b"),
    ("LINE_PARTNERSHIP_N_FUEL_13C_N", "Partnership {0} fuel 13c {1}"),
    ("LINE_PARTNERSHIP_N_FUEL_14A", "Partnership {0} fuel 14a"),
    ("LINE_PARTNERSHIP_N_FUEL_14B", "Partnership {0} fuel 14b"),
    ("LINE_PARTNERSHIP_N_FUEL_15A", "Partnership {0} fuel 15a"),
    ("LINE_PARTNERSHIP_N_FUEL_16A", "Partnership {0} fuel 16a"),
    ("LINE_PARTNERSHIP_N_FUEL_16B", "Partnership {0} fuel 16b"),
    ("LINE_PARTNERSHIP_N_FUEL_1A", "Partnership {0} fuel 1a"),
    ("LINE_PARTNERSHIP_N_FUEL_1B", "Partnership {0} fuel 1b"),
    ("LINE_PARTNERSHIP_N_FUEL_1C", "Partnership {0} fuel 1c"),
    ("LINE_PARTNERSHIP_N_FUEL_1D", "Partnership {0} fuel 1d"),
    ("LINE_PARTNERSHIP_N_FUEL_2A", "Partnership {0} fuel 2a"),
    ("LINE_PARTNERSHIP_N_FUEL_2B", "Partnership {0} fuel 2b"),
    ("LINE_PARTNERSHIP_N_FUEL_2C", "Partnership {0} fuel 2c"),
    ("LINE_PARTNERSHIP_N_FUEL_2D", "Partnership {0} fuel 2d"),
    ("LINE_PARTNERSHIP_N_FUEL_3A", "Partnership {0} fuel 3a"),
    ("LINE_PARTNERSHIP_N_FUEL_3B", "Partnership {0} fuel 3b"),
    ("LINE_PARTNERSHIP_N_FUEL_3C", "Partnership {0} fuel 3c"),
    ("LINE_PARTNERSHIP_N_FUEL_3D", "Partnership {0} fuel 3d"),
    ("LINE_PARTNERSHIP_N_FUEL_3E", "Partnership {0} fuel 3e"),
    ("LINE_PARTNERSHIP_N_FUEL_4A", "Partnership {0} fuel 4a"),
    ("LINE_PARTNERSHIP_N_FUEL_4B", "Partnership {0} fuel 4b"),
    ("LINE_PARTNERSHIP_N_FUEL_4C", "Partnership {0} fuel 4c"),
    ("LINE_PARTNERSHIP_N_FUEL_4D", "Partnership {0} fuel 4d"),
    ("LINE_PARTNERSHIP_N_FUEL_4E", "Partnership {0} fuel 4e"),
    ("LINE_PARTNERSHIP_N_FUEL_4F", "Partnership {0} fuel 4f"),
    ("LINE_PARTNERSHIP_N_FUEL_5A", "Partnership {0} fuel 5a"),
    ("LINE_PARTNERSHIP_N_FUEL_5B", "Partnership {0} fuel 5b"),
    ("LINE_PARTNERSHIP_N_FUEL_5C", "Partnership {0} fuel 5c"),
    ("LINE_PARTNERSHIP_N_FUEL_5D", "Partnership {0} fuel 5d"),
    ("LINE_PARTNERSHIP_N_FUEL_5E", "Partnership {0} fuel 5e"),
    ("LINE_PARTNERSHIP_N_FUEL_6A", "Partnership {0} fuel 6a"),
    ("LINE_PARTNERSHIP_N_FUEL_6B", "Partnership {0} fuel 6b"),
    ("LINE_PARTNERSHIP_N_FUEL_7A", "Partnership {0} fuel 7a"),
    ("LINE_PARTNERSHIP_N_FUEL_7B", "Partnership {0} fuel 7b"),
    ("LINE_PARTNERSHIP_N_FUEL_7C", "Partnership {0} fuel 7c"),
    ("LINE_PARTNERSHIP_N_FUEL_8A", "Partnership {0} fuel 8a"),
    ("LINE_PARTNERSHIP_N_FUEL_8B", "Partnership {0} fuel 8b"),
    ("LINE_PARTNERSHIP_N_FUEL_8C", "Partnership {0} fuel 8c"),
    ("LINE_PARTNERSHIP_N_FUEL_8D", "Partnership {0} fuel 8d"),
    ("LINE_PARTNERSHIP_N_FUEL_8E", "Partnership {0} fuel 8e"),
    ("LINE_PARTNERSHIP_N_FUEL_8F", "Partnership {0} fuel 8f"),
    ("LINE_PARTNERSHIP_N_INTEREST", "Partnership {0} interest"),
    ("LINE_PARTNERSHIP_N_LONG_GAIN", "Partnership {0} long gain"),
    ("LINE_PARTNERSHIP_N_PASSIVE_SUSPENDED", "Partnership {0} passive suspended"),
    (
        "LINE_PARTNERSHIP_N_SECTION179_PASSIVE_ALLOWED",
        "Partnership {0} section 179 allowed by the passive-loss limit",
    ),
    (
        "LINE_PARTNERSHIP_N_SECTION179_PASSIVE_UNALLOWED",
        "Partnership {0} section 179 passive unallowed carryover",
    ),
    (
        "LINE_SCORP_N_SECTION179_PASSIVE_ALLOWED",
        "S corporation {0} section 179 allowed by the passive-loss limit",
    ),
    (
        "LINE_SCORP_N_SECTION179_PASSIVE_UNALLOWED",
        "S corporation {0} section 179 passive unallowed carryover",
    ),
    (
        "LINE_AMT_PARTNERSHIP_N_SECTION179_PASSIVE_ALLOWED",
        "AMT partnership {0} section 179 allowed by the passive-loss limit",
    ),
    (
        "LINE_AMT_PARTNERSHIP_N_SECTION179_PASSIVE_UNALLOWED",
        "AMT partnership {0} section 179 passive unallowed carryover",
    ),
    (
        "LINE_AMT_SCORP_N_SECTION179_PASSIVE_ALLOWED",
        "AMT S corporation {0} section 179 allowed by the passive-loss limit",
    ),
    (
        "LINE_AMT_SCORP_N_SECTION179_PASSIVE_UNALLOWED",
        "AMT S corporation {0} section 179 passive unallowed carryover",
    ),
    ("LINE_PARTNERSHIP_N_PRIOR_FUEL_CREDIT_RECOVERY", "Partnership {0} prior fuel credit recovery"),
    ("LINE_PARTNERSHIP_N_SECTION179", "Partnership {0} section 179"),
    ("LINE_PARTNERSHIP_N_SECTION179_CARRYFORWARD", "Partnership {0} section 179 carryforward"),
    (
        "LINE_PARTNERSHIP_N_SECTION179_PRIOR_CARRYOVER",
        "Partnership {0} section 179 prior carryover",
    ),
    ("LINE_PARTNERSHIP_N_SHORT_GAIN", "Partnership {0} short gain"),
    ("LINE_PAYMENTS", "Payments"),
    ("LINE_PENSION_N_BASIS_RECOVERY", "Pension {0} basis recovery"),
    ("LINE_PENSION_N_COMMON_MONTHLY_EXCLUSION", "Pension {0} common monthly exclusion"),
    ("LINE_PENSION_N_EARLIER_COMMON_RECOVERY", "Pension {0} earlier common recovery"),
    ("LINE_PENSION_N_INCOME_EXCLUSION", "Pension {0} income exclusion"),
    ("LINE_PENSION_N_MONTHLY_EXCLUSION", "Pension {0} monthly exclusion"),
    ("LINE_PENSION_N_NEXT_YEAR_IRA_BASIS", "Pension {0} next year IRA basis"),
    ("LINE_PENSION_N_REMAINING_BASIS", "Pension {0} remaining basis"),
    ("LINE_PENSION_N_ROLLED_BASIS", "Pension {0} rolled basis"),
    ("LINE_PENSION_CONTRIBUTION_DEDUCTION", "Pension contribution deduction"),
    ("LINE_PENSION_CONTRIBUTION_LIMIT_SPOUSE", "Pension contribution limit, spouse"),
    ("LINE_PENSION_CONTRIBUTION_LIMIT_TAXPAYER", "Pension contribution limit, taxpayer"),
    ("LINE_PERSONAL_CASUALTY_DEDUCTION", "Personal casualty deduction"),
    ("LINE_PERSONAL_DEBT_CANCELLATION", "Personal debt cancellation"),
    ("LINE_PERSONAL_REFUELING_CREDIT", "Personal refueling credit"),
    ("LINE_PERSONAL_REFUELING_CREDIT_UNUSABLE", "Personal refueling credit unusable"),
    ("LINE_PERSONAL_RENT_N_INVESTMENT_INCOME", "Personal rent {0} investment income"),
    ("LINE_PERSONAL_RENTAL_N_ALLOWED_EXPENSES", "Personal rental {0} allowed expenses"),
    (
        "LINE_PERSONAL_RENTAL_N_ASSET_N_AMT_DEPRECIATION",
        "Personal rental {0} asset {1} AMT depreciation",
    ),
    (
        "LINE_PERSONAL_RENTAL_N_ASSET_N_AMT_REMAINING_BASIS",
        "Personal rental {0} asset {1} AMT remaining basis",
    ),
    ("LINE_PERSONAL_RENTAL_N_ASSET_N_BONUS", "Personal rental {0} asset {1} bonus"),
    ("LINE_PERSONAL_RENTAL_N_ASSET_N_MACRS", "Personal rental {0} asset {1} MACRS"),
    (
        "LINE_PERSONAL_RENTAL_N_ASSET_N_REMAINING_BASIS",
        "Personal rental {0} asset {1} remaining basis",
    ),
    (
        "LINE_PERSONAL_RENTAL_N_PRIOR_SUSPENDED_EXPENSES",
        "Personal rental {0} prior suspended expenses",
    ),
    ("LINE_PERSONAL_RENTAL_N_SUSPENDED_LOSS", "Personal rental {0} suspended loss"),
    ("LINE_PERSONAL_RENTAL_PASSIVE_INCOME", "Personal rental passive income"),
    ("LINE_PERSONAL_SALE_DISALLOWED_LOSS", "Personal sale disallowed loss"),
    ("LINE_PREMIUM_CREDIT_ADVANCE", "Premium credit advance"),
    ("LINE_PREMIUM_CREDIT_REFUNDABLE", "Premium credit refundable"),
    ("LINE_PREMIUM_CREDIT_REPAYMENT", "Premium credit repayment"),
    ("LINE_PREMIUM_TAX_CREDIT", "Premium tax credit"),
    ("LINE_PREVIOUSLY_OWNED_CLEAN_VEHICLE_CREDIT", "Previously owned clean vehicle credit"),
    (
        "LINE_PREVIOUSLY_OWNED_CLEAN_VEHICLE_CREDIT_UNUSABLE",
        "Previously owned clean vehicle credit unusable",
    ),
    (
        "LINE_PREVIOUSLY_OWNED_CLEAN_VEHICLE_TRANSFER_REPAYMENT",
        "Previously owned clean vehicle transfer repayment",
    ),
    ("LINE_PRIOR_ATNOL_DEDUCTION", "Prior ATNOL deduction (alternative tax net operating loss)"),
    ("LINE_PRIOR_INCOME_REPAYMENT_REVIEW_TOTAL", "Prior income repayment review total"),
    ("LINE_PRIOR_NOL_DEDUCTION", "Prior NOL deduction"),
    ("LINE_PRIOR_YEAR_CARE_CREDIT", "Prior year care credit"),
    (
        "LINE_PROPERTY_ROLLOVER_N_RETAINED_CAPITAL_GAIN",
        "Property rollover {0} retained capital gain",
    ),
    ("LINE_PROPERTY_ROLLOVER_N_RETAINED_PENSION", "Property rollover {0} retained pension"),
    ("LINE_PUBLIC_SAFETY_N_EXCLUDED", "Public safety {0} excluded"),
    ("LINE_PUBLIC_SAFETY_MEDICAL_BEFORE_COORDINATION", "Public safety medical before coordination"),
    ("LINE_QBI_ACTIVE_MINIMUM", "QBI active minimum"),
    ("LINE_QBI_ACTIVITY_N", "QBI activity {0}"),
    ("LINE_QBI_ACTIVITY_N_COMPONENT", "QBI activity {0} component"),
    ("LINE_QBI_ACTIVITY_N_TWENTY_PERCENT", "QBI activity {0} twenty percent"),
    ("LINE_QBI_AFTER_COOPERATIVE", "QBI after cooperative"),
    ("LINE_QBI_PATRON_REDUCTION", "QBI patron reduction"),
    ("LINE_QBI_BEFORE_INCOME_LIMIT", "QBI before income limit"),
    ("LINE_QBI_COOPERATIVE_DPAD", "Cooperative domestic production deduction"),
    ("LINE_QBI_ACTIVITY_N_PATRON_INCOME", "Activity {0} qualified-payment QBI"),
    (
        "LINE_QBI_ACTIVITY_N_PATRON_NINE_PERCENT",
        "Activity {0} nine percent of qualified-payment QBI",
    ),
    ("LINE_QBI_ACTIVITY_N_PATRON_WAGES", "Activity {0} qualified-payment W-2 wages"),
    ("LINE_QBI_ACTIVITY_N_PATRON_HALF_WAGES", "Activity {0} half qualified-payment W-2 wages"),
    ("LINE_QBI_COMPONENT", "QBI component"),
    ("LINE_QBI_DEDUCTION", "QBI deduction"),
    ("LINE_QBI_INCOME_LESS_GAIN", "QBI income less gain"),
    ("LINE_QBI_INCOME_LIMIT", "QBI income limit"),
    ("LINE_QBI_LIMITED_DEDUCTION", "QBI limited deduction"),
    ("LINE_QBI_LOSS_CARRYFORWARD", "QBI loss carryforward"),
    ("LINE_QBI_NET", "QBI net"),
    ("LINE_QBI_NET_CAPITAL_GAIN", "QBI net capital gain"),
    ("LINE_QBI_PRIOR_LOSS", "QBI prior loss"),
    ("LINE_QBI_PRIOR_REIT_PTP_LOSS", "QBI prior REIT PTP loss (publicly traded partnership)"),
    ("LINE_QBI_REIT_PTP", "QBI REIT PTP (publicly traded partnership)"),
    ("LINE_QBI_REIT_PTP_COMPONENT", "QBI REIT PTP component (publicly traded partnership)"),
    ("LINE_QBI_REIT_PTP_NET", "QBI REIT PTP net (publicly traded partnership)"),
    ("LINE_QBI_TAXABLE_INCOME", "QBI taxable income"),
    ("LINE_QBI_TOTAL", "QBI total"),
    (
        "LINE_QCD_ADJUSTMENT_CARRYFORWARD",
        "QCD adjustment carryforward (qualified charitable distributions)",
    ),
    ("LINE_QCD_CASH_GIFTS", "QCD cash gifts (qualified charitable distributions)"),
    ("LINE_QCD_EXCLUSION", "QCD exclusion (qualified charitable distributions)"),
    (
        "LINE_QCD_SPOUSE_ADJUSTMENT_CARRY",
        "QCD adjustment carry, spouse (qualified charitable distributions)",
    ),
    ("LINE_QCD_SPOUSE_EXCLUSION", "QCD exclusion, spouse (qualified charitable distributions)"),
    (
        "LINE_QCD_TAXPAYER_ADJUSTMENT_CARRY",
        "QCD adjustment carry, taxpayer (qualified charitable distributions)",
    ),
    ("LINE_QCD_TAXPAYER_EXCLUSION", "QCD exclusion, taxpayer (qualified charitable distributions)"),
    ("LINE_QSBS_N_AMT_PREFERENCE", "QSBS {0} AMT preference (qualified small business stock)"),
    ("LINE_QSBS_N_EXCLUDED_GAIN", "QSBS {0} excluded gain (qualified small business stock)"),
    ("LINE_QSBS_N_EXCLUSION_LIMIT", "QSBS {0} exclusion limit (qualified small business stock)"),
    ("LINE_QSBS_N_RATE_N_GAIN", "QSBS {0} rate {1} gain (qualified small business stock)"),
    ("LINE_QSBS_N_REALIZED_GAIN", "QSBS {0} realized gain (qualified small business stock)"),
    ("LINE_QSBS_N_TAXABLE_GAIN", "QSBS {0} taxable gain (qualified small business stock)"),
    (
        "LINE_QSBS_N_ZONE_GAIN_THROUGH_N",
        "QSBS {0} zone gain through {1} (qualified small business stock)",
    ),
    ("LINE_RAILROAD_PRIOR_YEAR_REPAYMENTS", "Railroad prior year repayments"),
    ("LINE_RATE_N_GAIN", "Rate {0} gain"),
    ("LINE_REFUNDABLE_EDUCATION_CREDIT", "Refundable education credit"),
    ("LINE_REGULAR_INCOME_TAX", "Regular income tax"),
    ("LINE_REGULAR_TOTAL_NOL_CARRYFORWARD", "Regular total NOL carryforward"),
    ("LINE_REIT_PTP_LOSS_CARRYFORWARD", "REIT PTP loss carryforward (publicly traded partnership)"),
    ("LINE_RENTAL_N_BUILDING_N_DEPRECIATION", "Rental {0} building {1} depreciation"),
    ("LINE_RENTAL_N_DEPRECIATION", "Rental {0} depreciation"),
    ("LINE_RENTAL_N_EXPENSE_N_DE_MINIMIS", "Rental {0} expense {1} de minimis"),
    (
        "LINE_RENTAL_N_EXPENSE_LINE10_LEGAL_AND_PROFESSIONAL",
        "Rental {0} expense line 10 legal and professional",
    ),
    ("LINE_RENTAL_N_EXPENSE_LINE11_MANAGEMENT_FEES", "Rental {0} expense line 11 management fees"),
    (
        "LINE_RENTAL_N_EXPENSE_LINE12_MORTGAGE_INTEREST",
        "Rental {0} expense line 12 mortgage interest",
    ),
    (
        "LINE_RENTAL_N_EXPENSE_LINE13A_VEHICLE_LOAN_INTEREST",
        "Rental {0} expense line 13a vehicle loan interest",
    ),
    ("LINE_RENTAL_N_EXPENSE_LINE13B_OTHER_INTEREST", "Rental {0} expense line 13b other interest"),
    ("LINE_RENTAL_N_EXPENSE_LINE14_REPAIRS", "Rental {0} expense line 14 repairs"),
    ("LINE_RENTAL_N_EXPENSE_LINE15_SUPPLIES", "Rental {0} expense line 15 supplies"),
    ("LINE_RENTAL_N_EXPENSE_LINE16_TAXES", "Rental {0} expense line 16 taxes"),
    ("LINE_RENTAL_N_EXPENSE_LINE17_UTILITIES", "Rental {0} expense line 17 utilities"),
    ("LINE_RENTAL_N_EXPENSE_LINE19_OTHER", "Rental {0} expense line 19 other"),
    ("LINE_RENTAL_N_EXPENSE_LINE5_ADVERTISING", "Rental {0} expense line 5 advertising"),
    ("LINE_RENTAL_N_EXPENSE_LINE6_AUTO_AND_TRAVEL", "Rental {0} expense line 6 auto and travel"),
    (
        "LINE_RENTAL_N_EXPENSE_LINE7_CLEANING_AND_MAINTENANCE",
        "Rental {0} expense line 7 cleaning and maintenance",
    ),
    ("LINE_RENTAL_N_EXPENSE_LINE8_COMMISSIONS", "Rental {0} expense line 8 commissions"),
    ("LINE_RENTAL_N_EXPENSE_LINE9_INSURANCE", "Rental {0} expense line 9 insurance"),
    (
        "LINE_RENTAL_N_HOME_DEPRECIATION_CARRYOVER",
        "Rental {0} home casualty and depreciation carryover",
    ),
    ("LINE_RENTAL_N_HOME_OPERATING_CARRYOVER", "Rental {0} home operating carryover"),
    ("LINE_RENTAL_N_LINE18_DEPRECIATION", "Rental {0} line 18 depreciation"),
    ("LINE_RENTAL_N_LINE20_TOTAL_EXPENSES", "Rental {0} line 20 total expenses"),
    ("LINE_RENTAL_N_LINE21_INCOME", "Rental {0} line 21 income"),
    ("LINE_RENTAL_N_LINE22_ALLOWED_LOSS", "Rental {0} line 22 allowed loss"),
    ("LINE_RENTAL_N_LINE3_RENTS", "Rental {0} line 3 rents"),
    ("LINE_RENTAL_N_SALE_BUILDING_ADJUSTED_BASIS", "Rental {0} sale building adjusted basis"),
    ("LINE_RENTAL_N_SALE_BUILDING_COST", "Rental {0} sale building cost"),
    ("LINE_RENTAL_N_SALE_BUILDING_DEPRECIATION", "Rental {0} sale building depreciation"),
    ("LINE_RENTAL_N_SALE_BUILDING_GAIN", "Rental {0} sale building gain"),
    ("LINE_RENTAL_N_SALE_BUILDING_SALE_PRICE", "Rental {0} sale building sale price"),
    ("LINE_RENTAL_N_SALE_FORM6252_ADJUSTED_BASIS", "Rental {0} sale Form 6252 adjusted basis"),
    ("LINE_RENTAL_N_SALE_FORM6252_ASSUMED_DEBT", "Rental {0} sale Form 6252 assumed debt"),
    ("LINE_RENTAL_N_SALE_FORM6252_CONTRACT_PRICE", "Rental {0} sale Form 6252 contract price"),
    ("LINE_RENTAL_N_SALE_FORM6252_COST", "Rental {0} sale Form 6252 cost"),
    ("LINE_RENTAL_N_SALE_FORM6252_DEEMED_PAYMENT", "Rental {0} sale Form 6252 deemed payment"),
    ("LINE_RENTAL_N_SALE_FORM6252_DEPRECIATION", "Rental {0} sale Form 6252 depreciation"),
    ("LINE_RENTAL_N_SALE_FORM6252_EXCESS_DEBT", "Rental {0} sale Form 6252 excess debt"),
    ("LINE_RENTAL_N_SALE_FORM6252_EXPENSES", "Rental {0} sale Form 6252 expenses"),
    ("LINE_RENTAL_N_SALE_FORM6252_GAIN", "Rental {0} sale Form 6252 gain"),
    ("LINE_RENTAL_N_SALE_FORM6252_GROSS_PROFIT", "Rental {0} sale Form 6252 gross profit"),
    ("LINE_RENTAL_N_SALE_FORM6252_PROFIT", "Rental {0} sale Form 6252 profit before exclusion"),
    ("LINE_RENTAL_N_SALE_FORM6252_PERCENTAGE", "Rental {0} sale Form 6252 gross profit percentage"),
    ("LINE_RENTAL_N_SALE_FORM6252_INCOME", "Rental {0} sale Form 6252 income"),
    ("LINE_RENTAL_N_SALE_FORM6252_NET_PRICE", "Rental {0} sale Form 6252 net price"),
    ("LINE_RENTAL_N_SALE_FORM6252_ORDINARY", "Rental {0} sale Form 6252 ordinary"),
    ("LINE_RENTAL_N_SALE_FORM6252_PAYMENTS", "Rental {0} sale Form 6252 payments"),
    ("LINE_RENTAL_N_SALE_FORM6252_PRICE", "Rental {0} sale Form 6252 price"),
    ("LINE_RENTAL_N_SALE_FORM6252_RECAPTURE", "Rental {0} sale Form 6252 recapture"),
    ("LINE_RENTAL_N_SALE_FORM6252_TOTAL", "Rental {0} sale Form 6252 total"),
    ("LINE_RENTAL_N_SALE_FORM6252_TOTAL_PAYMENTS", "Rental {0} sale Form 6252 total payments"),
    ("LINE_RENTAL_N_SALE_FORM6252_UNRECAPTURED_N", "Rental {0} sale Form 6252 unrecaptured {1}"),
    ("LINE_RENTAL_N_SALE_INSTALLMENT_GAIN", "Rental {0} sale installment gain"),
    ("LINE_RENTAL_N_SALE_LAND", "Rental {0} sale land"),
    ("LINE_RENTAL_N_SALE_UNRECAPTURED_N", "Rental {0} sale unrecaptured {1}"),
    ("LINE_RENTAL_N_SUSPENDED_LOSS", "Rental {0} suspended loss"),
    ("LINE_RENTAL_INCOME", "Rental income"),
    ("LINE_RENTAL_MAGI", "Rental MAGI"),
    ("LINE_RENTAL_SUSPENDED_LOSS", "Rental suspended loss"),
    ("LINE_REPAYMENT_N_DECREASE", "Repayment {0} decrease"),
    ("LINE_REPAYMENT_N_ORIGINAL_CHAPTER_ONE_NET_TAX", "Repayment {0} original chapter one net tax"),
    (
        "LINE_REPAYMENT_N_REFIGURED_CHAPTER_ONE_NET_TAX",
        "Repayment {0} refigured chapter one net tax",
    ),
    ("LINE_REPAYMENT_CREDIT", "Repayment credit"),
    ("LINE_REPAYMENT_CREDIT_POTENTIAL", "Repayment credit potential"),
    ("LINE_REPAYMENT_ITEMIZED_AMOUNT", "Repayment itemized amount"),
    ("LINE_REPAYMENT_YEAR_N_CAPITAL_DISTRIBUTIONS", "Repayment year {0} capital distributions"),
    ("LINE_REPAYMENT_YEAR_N_CAPITAL_INCOME", "Repayment year {0} capital income"),
    ("LINE_REPAYMENT_YEAR_N_COLLECTIBLES_GAIN", "Repayment year {0} collectibles gain"),
    ("LINE_REPAYMENT_YEAR_N_DECREASE", "Repayment year {0} decrease"),
    ("LINE_REPAYMENT_YEAR_N_IRA_GROSS", "Repayment year {0} IRA gross"),
    ("LINE_REPAYMENT_YEAR_N_IRA_TAXABLE", "Repayment year {0} IRA taxable"),
    (
        "LINE_REPAYMENT_YEAR_N_MINIMUM_CREDIT_AVAILABLE",
        "Repayment year {0} minimum credit available",
    ),
    (
        "LINE_REPAYMENT_YEAR_N_MINIMUM_CREDIT_GENERATED",
        "Repayment year {0} minimum credit generated",
    ),
    ("LINE_REPAYMENT_YEAR_N_MINIMUM_EXCLUSION_TAX", "Repayment year {0} minimum exclusion tax"),
    ("LINE_REPAYMENT_YEAR_N_NET_LONG_CAPITAL", "Repayment year {0} net long capital"),
    ("LINE_REPAYMENT_YEAR_N_NET_SHORT_CAPITAL", "Repayment year {0} net short capital"),
    ("LINE_REPAYMENT_YEAR_N_ORDINARY_DIVIDENDS", "Repayment year {0} ordinary dividends"),
    ("LINE_REPAYMENT_YEAR_N_ORIGINAL_AGI", "Repayment year {0} original AGI"),
    ("LINE_REPAYMENT_YEAR_N_ORIGINAL_AMT", "Repayment year {0} original AMT"),
    (
        "LINE_REPAYMENT_YEAR_N_ORIGINAL_AMT_CAPITAL_INCOME",
        "Repayment year {0} original AMT capital income",
    ),
    (
        "LINE_REPAYMENT_YEAR_N_ORIGINAL_AMT_LONG_CAPITAL",
        "Repayment year {0} original AMT long capital",
    ),
    ("LINE_REPAYMENT_YEAR_N_ORIGINAL_AMT_LONG_CARRY", "Repayment year {0} original AMT long carry"),
    (
        "LINE_REPAYMENT_YEAR_N_ORIGINAL_AMT_SHORT_CAPITAL",
        "Repayment year {0} original AMT short capital",
    ),
    (
        "LINE_REPAYMENT_YEAR_N_ORIGINAL_AMT_SHORT_CARRY",
        "Repayment year {0} original AMT short carry",
    ),
    ("LINE_REPAYMENT_YEAR_N_ORIGINAL_AMTI", "Repayment year {0} original AMTI"),
    ("LINE_REPAYMENT_YEAR_N_ORIGINAL_EARLY_TAX", "Repayment year {0} original early tax"),
    ("LINE_REPAYMENT_YEAR_N_ORIGINAL_EIC", "Repayment year {0} original EIC"),
    ("LINE_REPAYMENT_YEAR_N_ORIGINAL_ELDERLY_CREDIT", "Repayment year {0} original elderly credit"),
    (
        "LINE_REPAYMENT_YEAR_N_ORIGINAL_EXCLUSION_INCOME",
        "Repayment year {0} original exclusion income",
    ),
    (
        "LINE_REPAYMENT_YEAR_N_ORIGINAL_EXCLUSION_LONG_CARRY",
        "Repayment year {0} original exclusion long carry",
    ),
    (
        "LINE_REPAYMENT_YEAR_N_ORIGINAL_EXCLUSION_SHORT_CARRY",
        "Repayment year {0} original exclusion short carry",
    ),
    ("LINE_REPAYMENT_YEAR_N_ORIGINAL_LONG_CARRY", "Repayment year {0} original long carry"),
    ("LINE_REPAYMENT_YEAR_N_ORIGINAL_MINIMUM_CREDIT", "Repayment year {0} original minimum credit"),
    (
        "LINE_REPAYMENT_YEAR_N_ORIGINAL_MINIMUM_CREDIT_CARRY",
        "Repayment year {0} original minimum credit carry",
    ),
    (
        "LINE_REPAYMENT_YEAR_N_ORIGINAL_SENIOR_DEDUCTION",
        "Repayment year {0} original senior deduction",
    ),
    ("LINE_REPAYMENT_YEAR_N_ORIGINAL_SHORT_CARRY", "Repayment year {0} original short carry"),
    ("LINE_REPAYMENT_YEAR_N_ORIGINAL_TAX", "Repayment year {0} original tax"),
    ("LINE_REPAYMENT_YEAR_N_ORIGINAL_TAXABLE_SOCIAL", "Repayment year {0} original taxable social"),
    (
        "LINE_REPAYMENT_YEAR_N_ORIGINAL_TENTATIVE_MINIMUM_TAX",
        "Repayment year {0} original tentative minimum tax",
    ),
    ("LINE_REPAYMENT_YEAR_N_PENSION_GROSS", "Repayment year {0} pension gross"),
    ("LINE_REPAYMENT_YEAR_N_PENSION_TAXABLE", "Repayment year {0} pension taxable"),
    ("LINE_REPAYMENT_YEAR_N_QUALIFIED_DIVIDENDS", "Repayment year {0} qualified dividends"),
    ("LINE_REPAYMENT_YEAR_N_REFIGURED_AGI", "Repayment year {0} refigured AGI"),
    ("LINE_REPAYMENT_YEAR_N_REFIGURED_AMT", "Repayment year {0} refigured AMT"),
    (
        "LINE_REPAYMENT_YEAR_N_REFIGURED_AMT_CAPITAL_INCOME",
        "Repayment year {0} refigured AMT capital income",
    ),
    (
        "LINE_REPAYMENT_YEAR_N_REFIGURED_AMT_LONG_CARRY",
        "Repayment year {0} refigured AMT long carry",
    ),
    (
        "LINE_REPAYMENT_YEAR_N_REFIGURED_AMT_SHORT_CARRY",
        "Repayment year {0} refigured AMT short carry",
    ),
    ("LINE_REPAYMENT_YEAR_N_REFIGURED_AMTI", "Repayment year {0} refigured AMTI"),
    (
        "LINE_REPAYMENT_YEAR_N_REFIGURED_CAPITAL_INCOME",
        "Repayment year {0} refigured capital income",
    ),
    ("LINE_REPAYMENT_YEAR_N_REFIGURED_EARLY_TAX", "Repayment year {0} refigured early tax"),
    ("LINE_REPAYMENT_YEAR_N_REFIGURED_EIC", "Repayment year {0} refigured EIC"),
    (
        "LINE_REPAYMENT_YEAR_N_REFIGURED_ELDERLY_CREDIT",
        "Repayment year {0} refigured elderly credit",
    ),
    (
        "LINE_REPAYMENT_YEAR_N_REFIGURED_EXCLUSION_INCOME",
        "Repayment year {0} refigured exclusion income",
    ),
    (
        "LINE_REPAYMENT_YEAR_N_REFIGURED_EXCLUSION_LONG_CARRY",
        "Repayment year {0} refigured exclusion long carry",
    ),
    (
        "LINE_REPAYMENT_YEAR_N_REFIGURED_EXCLUSION_SHORT_CARRY",
        "Repayment year {0} refigured exclusion short carry",
    ),
    ("LINE_REPAYMENT_YEAR_N_REFIGURED_LONG_CARRY", "Repayment year {0} refigured long carry"),
    (
        "LINE_REPAYMENT_YEAR_N_REFIGURED_MINIMUM_CREDIT",
        "Repayment year {0} refigured minimum credit",
    ),
    (
        "LINE_REPAYMENT_YEAR_N_REFIGURED_MINIMUM_CREDIT_AVAILABLE",
        "Repayment year {0} refigured minimum credit available",
    ),
    (
        "LINE_REPAYMENT_YEAR_N_REFIGURED_MINIMUM_CREDIT_CARRY",
        "Repayment year {0} refigured minimum credit carry",
    ),
    (
        "LINE_REPAYMENT_YEAR_N_REFIGURED_RETIREMENT_TAXABLE",
        "Repayment year {0} refigured retirement taxable",
    ),
    (
        "LINE_REPAYMENT_YEAR_N_REFIGURED_SENIOR_DEDUCTION",
        "Repayment year {0} refigured senior deduction",
    ),
    ("LINE_REPAYMENT_YEAR_N_REFIGURED_SHORT_CARRY", "Repayment year {0} refigured short carry"),
    ("LINE_REPAYMENT_YEAR_N_REFIGURED_TAX", "Repayment year {0} refigured tax"),
    (
        "LINE_REPAYMENT_YEAR_N_REFIGURED_TAXABLE_SOCIAL",
        "Repayment year {0} refigured taxable social",
    ),
    (
        "LINE_REPAYMENT_YEAR_N_REFIGURED_TENTATIVE_MINIMUM_TAX",
        "Repayment year {0} refigured tentative minimum tax",
    ),
    ("LINE_REPAYMENT_YEAR_N_RETIREMENT_TAXABLE", "Repayment year {0} retirement taxable"),
    ("LINE_REPAYMENT_YEAR_N_SOCIAL_BENEFITS", "Repayment year {0} social benefits"),
    ("LINE_REPAYMENT_YEAR_N_UNRECAPTURED_GAIN", "Repayment year {0} unrecaptured gain"),
    ("LINE_REQUIRED_DISTRIBUTION_TAX", "Required distribution tax"),
    ("LINE_RESERVIST_SPOUSE_N_BASIS", "Reservist {0} basis, spouse"),
    ("LINE_RESERVIST_SPOUSE_N_UNREPAID", "Reservist {0} unrepaid, spouse"),
    ("LINE_RESERVIST_TAXPAYER_N_BASIS", "Reservist {0} basis, taxpayer"),
    ("LINE_RESERVIST_TAXPAYER_N_UNREPAID", "Reservist {0} unrepaid, taxpayer"),
    ("LINE_RESIDENTIAL_ENERGY_CARRYFORWARD_N", "Residential energy carryforward {0}"),
    ("LINE_RESIDENTIAL_ENERGY_CREDIT", "Residential energy credit"),
    ("LINE_RETIREMENT_N_QUALIFIED_GROSS", "Retirement {0} qualified gross"),
    ("LINE_RETIREMENT_N_QUALIFIED_TAXABLE", "Retirement {0} qualified taxable"),
    ("LINE_RETIREMENT_ROLLOVERS", "Retirement rollovers"),
    ("LINE_RMD_N_ACCOUNT_N_MINIMUM", "RMD {0} account {1} minimum (required minimum distribution)"),
    (
        "LINE_RMD_N_CONDITIONAL_REDUCED_SHORTFALL",
        "RMD {0} conditional reduced shortfall (required minimum distribution)",
    ),
    ("LINE_RMD_N_DEFERRED_TO_N", "RMD {0} deferred to {1} (required minimum distribution)"),
    ("LINE_RMD_N_INHERITED_MINIMUM", "RMD {0} inherited minimum (required minimum distribution)"),
    ("LINE_RMD_N_SHORTFALL", "RMD {0} shortfall (required minimum distribution)"),
    ("LINE_RMD_N_TAX", "RMD {0} tax (required minimum distribution)"),
    ("LINE_RMD_N_WAIVER_REQUESTED", "RMD {0} waiver requested (required minimum distribution)"),
    ("LINE_ROTH_N_CONVERSION_RECAPTURE", "Roth {0} conversion recapture"),
    ("LINE_ROTH_N_REGULAR_RECOVERED", "Roth {0} regular recovered"),
    ("LINE_ROTH_N_TAXABLE", "Roth {0} taxable"),
    ("LINE_ROTH_CONTRIBUTION_MAGI", "Roth contribution MAGI"),
    ("LINE_ROTH_CONVERSION_TAXABLE", "Roth conversion taxable"),
    ("LINE_ROYALTY_N_LINE4_ROYALTIES", "Royalty {0} line 4 royalties"),
    ("LINE_SAVER_AUTOMATIC_WITHDRAWALS_SPOUSE", "Saver automatic withdrawals, spouse"),
    ("LINE_SAVER_AUTOMATIC_WITHDRAWALS_TAXPAYER", "Saver automatic withdrawals, taxpayer"),
    ("LINE_SAVER_CREDIT", "Saver credit"),
    ("LINE_SAVINGS_BOND_EXCLUSION", "Excludable savings bond interest"),
    ("LINE_SAVINGS_WITHDRAWAL_PENALTY", "Savings withdrawal penalty"),
    ("LINE_QSBS_AMT_PREFERENCE", "Section 1202 AMT preference"),
    ("LINE_SCHEDULE_N_ADJUSTMENTS", "Schedule {0} adjustments"),
    (
        "LINE_SCHEDULE_N_PREMIUM_CREDIT_REPAYMENT",
        "Schedule {0} excess advance premium tax credit repayment",
    ),
    (
        "LINE_SCHEDULE_N_NEW_CLEAN_VEHICLE_REPAYMENT",
        "Schedule {0} new clean vehicle credit repayment",
    ),
    (
        "LINE_SCHEDULE_N_USED_CLEAN_VEHICLE_REPAYMENT",
        "Schedule {0} used clean vehicle credit repayment",
    ),
    ("LINE_SCHEDULE_N_CREDIT_REPAYMENTS_TOTAL", "Schedule {0} credit repayments total"),
    ("LINE_SCHEDULE_N_ALTERNATIVE_MINIMUM_TAX", "Schedule {0} alternative minimum tax"),
    ("LINE_SCHEDULE_N_PART_ONE_TOTAL", "Schedule {0} part I total"),
    ("LINE_SCHEDULE_N_TAX_FAVORED_ACCOUNT_TAXES", "Schedule {0} tax-favored account taxes"),
    ("LINE_SCHEDULE_N_MORTGAGE_SUBSIDY_RECAPTURE", "Schedule {0} mortgage subsidy recapture"),
    ("LINE_SCHEDULE_N_HSA_DISTRIBUTION_TAX", "Schedule {0} HSA distribution tax"),
    ("LINE_SCHEDULE_N_HSA_TESTING_PERIOD_TAX", "Schedule {0} HSA testing period tax"),
    ("LINE_SCHEDULE_N_ARCHER_MSA_TAX", "Schedule {0} Archer MSA tax"),
    ("LINE_SCHEDULE_N_MEDICARE_MSA_TAX", "Schedule {0} Medicare Advantage MSA tax"),
    ("LINE_SCHEDULE_N_OTHER_TAXES_TOTAL", "Schedule {0} other taxes total"),
    (
        "LINE_SCHEDULE_N_INCOME_AND_OTHER_TAXES_SUBTOTAL",
        "Schedule {0} income and other taxes subtotal",
    ),
    ("LINE_SCHEDULE_N_UNREPORTED_TIP_TAX", "Schedule {0} unreported tip tax"),
    ("LINE_SCHEDULE_N_UNCOLLECTED_WAGE_TAX", "Schedule {0} uncollected wage tax"),
    ("LINE_SCHEDULE_N_PAYROLL_TAX_SUBTOTAL", "Schedule {0} payroll tax subtotal"),
    ("LINE_SCHEDULE_N_HOUSEHOLD_EMPLOYMENT_TAX", "Schedule {0} household employment tax"),
    ("LINE_SCHEDULE_N_UNCOLLECTED_TIP_TAX", "Schedule {0} uncollected tip tax"),
    ("LINE_SCHEDULE_N_EMPLOYMENT_TAX_SUBTOTAL", "Schedule {0} employment tax subtotal"),
    ("LINE_SCHEDULE_N_EXCESS_CONTRIBUTION_TAXES", "Schedule {0} excess contribution taxes"),
    ("LINE_SCHEDULE_N_EMPLOYMENT_AND_OTHER_TOTAL", "Schedule {0} employment and other taxes total"),
    ("LINE_SCHEDULE_N_TOTAL_ADDITIONAL_TAXES", "Schedule {0} total additional taxes"),
    ("LINE_SCHEDULE_N_FOREIGN_TAX_CREDIT", "Schedule {0} foreign tax credit"),
    ("LINE_SCHEDULE_N_CARE_CREDIT", "Schedule {0} care credit"),
    ("LINE_SCHEDULE_N_EDUCATION_CREDIT", "Schedule {0} education credit"),
    ("LINE_SCHEDULE_N_SAVER_CREDIT", "Schedule {0} saver's credit"),
    (
        "LINE_SCHEDULE_N_RESIDENTIAL_CLEAN_ENERGY_CREDIT",
        "Schedule {0} residential clean energy credit",
    ),
    ("LINE_SCHEDULE_N_GENERAL_BUSINESS_CREDIT", "Schedule {0} general business credit"),
    ("LINE_SCHEDULE_N_MINIMUM_TAX_CREDIT", "Schedule {0} prior year minimum tax credit"),
    ("LINE_SCHEDULE_N_ADOPTION_CREDIT", "Schedule {0} adoption credit"),
    ("LINE_SCHEDULE_N_ELDERLY_DISABLED_CREDIT", "Schedule {0} elderly or disabled credit"),
    ("LINE_SCHEDULE_N_NEW_CLEAN_VEHICLE_CREDIT", "Schedule {0} new clean vehicle credit"),
    ("LINE_SCHEDULE_N_MORTGAGE_INTEREST_CREDIT", "Schedule {0} mortgage interest credit"),
    ("LINE_SCHEDULE_N_DC_HOMEBUYER_CREDIT", "Schedule {0} DC first-time homebuyer credit"),
    ("LINE_SCHEDULE_N_ELECTRIC_VEHICLE_CREDIT", "Schedule {0} qualified electric vehicle credit"),
    ("LINE_SCHEDULE_N_REFUELING_PROPERTY_CREDIT", "Schedule {0} refueling property credit"),
    ("LINE_SCHEDULE_N_TAX_CREDIT_BOND_CREDIT", "Schedule {0} tax credit bond credit"),
    (
        "LINE_SCHEDULE_N_USED_CLEAN_VEHICLE_CREDIT",
        "Schedule {0} previously owned clean vehicle credit",
    ),
    ("LINE_SCHEDULE_N_OTHER_NONREFUNDABLE_TOTAL", "Schedule {0} other nonrefundable credits total"),
    ("LINE_SCHEDULE_N_NONREFUNDABLE_TOTAL", "Schedule {0} nonrefundable credits total"),
    ("LINE_SCHEDULE_N_NET_PREMIUM_TAX_CREDIT", "Schedule {0} net premium tax credit"),
    ("LINE_SCHEDULE_N_EXTENSION_PAYMENT", "Schedule {0} extension payment"),
    ("LINE_SCHEDULE_N_EXCESS_SOCIAL_SECURITY", "Schedule {0} excess social security tax"),
    ("LINE_SCHEDULE_N_FUEL_TAX_CREDIT", "Schedule {0} fuel tax credit"),
    ("LINE_SCHEDULE_N_REPAYMENT_TAX_CREDIT", "Schedule {0} repayment tax credit"),
    ("LINE_SCHEDULE_N_INSTALLMENT_DEFERRAL_PAYMENT", "Schedule {0} installment deferral payment"),
    ("LINE_SCHEDULE_N_OTHER_PAYMENTS_TOTAL", "Schedule {0} other payments total"),
    ("LINE_SCHEDULE_N_REFUNDABLE_TOTAL", "Schedule {0} refundable total"),
    ("LINE_SCHEDULE_N_TAXABLE_REFUNDS", "Schedule {0} taxable refunds"),
    ("LINE_SCHEDULE_N_ALIMONY_RECEIVED", "Schedule {0} alimony received"),
    ("LINE_SCHEDULE_N_OTHER_GAINS", "Schedule {0} other gains or losses"),
    ("LINE_SCHEDULE_N_UNEMPLOYMENT", "Schedule {0} unemployment compensation"),
    ("LINE_SCHEDULE_N_NET_OPERATING_LOSS", "Schedule {0} net operating loss"),
    ("LINE_SCHEDULE_N_CANCELED_DEBT", "Schedule {0} canceled debt"),
    (
        "LINE_SCHEDULE_N_FOREIGN_EARNED_INCOME_EXCLUSION",
        "Schedule {0} foreign earned income exclusion",
    ),
    ("LINE_SCHEDULE_N_MEDICAL_SAVINGS_INCOME", "Schedule {0} medical savings income"),
    ("LINE_SCHEDULE_N_HSA_INCOME", "Schedule {0} HSA income"),
    ("LINE_SCHEDULE_N_PRIZES_AND_AWARDS", "Schedule {0} prizes and awards"),
    ("LINE_SCHEDULE_N_STOCK_OPTIONS", "Schedule {0} stock options"),
    ("LINE_SCHEDULE_N_PERSONAL_PROPERTY_RENTAL", "Schedule {0} personal property rental"),
    ("LINE_SCHEDULE_N_EXCESS_BUSINESS_LOSS", "Schedule {0} excess business loss"),
    ("LINE_SCHEDULE_N_ABLE_DISTRIBUTIONS", "Schedule {0} ABLE distributions"),
    ("LINE_SCHEDULE_N_TOTAL_OTHER_INCOME", "Schedule {0} total other income"),
    ("LINE_SCHEDULE_N_ARCHER_MSA_DEDUCTION", "Schedule {0} Archer MSA deduction"),
    (
        "LINE_SCHEDULE_N_PERSONAL_PROPERTY_RENTAL_EXPENSES",
        "Schedule {0} personal property rental expenses",
    ),
    ("LINE_SCHEDULE_N_PENSION_CONTRIBUTIONS", "Schedule {0} pension contributions"),
    ("LINE_SCHEDULE_N_HOUSING_DEDUCTION", "Schedule {0} housing deduction"),
    ("LINE_SCHEDULE_N_ESTATE_DEDUCTIONS", "Schedule {0} estate deductions"),
    ("LINE_SCHEDULE_N_TOTAL_OTHER_ADJUSTMENTS", "Schedule {0} total other adjustments"),
    ("LINE_SCHEDULE_N_MOVING_EXPENSES", "Schedule {0} moving expenses"),
    ("LINE_SCHEDULE_N_ALIMONY_PAID", "Schedule {0} alimony paid"),
    ("LINE_SCHEDULE_N_BUSINESS_INCOME", "Schedule {0} business income"),
    ("LINE_SCHEDULE_N_EARLY_WITHDRAWAL_PENALTY", "Schedule {0} early withdrawal penalty"),
    ("LINE_SCHEDULE_N_EDUCATOR_EXPENSES", "Schedule {0} educator expenses"),
    ("LINE_SCHEDULE_N_FARM_INCOME", "Schedule {0} farm income"),
    ("LINE_SCHEDULE_N_HALF_SELF_EMPLOYMENT_TAX", "Schedule {0} half self-employment tax"),
    ("LINE_SCHEDULE_N_HSA_DEDUCTION", "Schedule {0} HSA deduction"),
    ("LINE_SCHEDULE_N_IRA_DEDUCTION", "Schedule {0} IRA deduction"),
    ("LINE_SCHEDULE_N_LINE_13Z_FORM_N", "Schedule {0} line 13z Form {1}"),
    ("LINE_SCHEDULE_N_LINE_6L", "Schedule {0} line 6l"),
    ("LINE_SCHEDULE_N_OTHER_INCOME", "Schedule {0} other income"),
    ("LINE_SCHEDULE_N_RENTAL_INCOME", "Schedule {0} rental income"),
    (
        "LINE_SCHEDULE_N_SELF_EMPLOYED_HEALTH_INSURANCE",
        "Schedule {0} self employed health insurance",
    ),
    ("LINE_SCHEDULE_N_SELF_EMPLOYED_RETIREMENT", "Schedule {0} self employed retirement"),
    ("LINE_SCHEDULE_N_STUDENT_LOAN_INTEREST", "Schedule {0} student loan interest"),
    ("LINE_SCHEDULE_AI_FULL_YEAR_ITEMIZED", "Schedule AI full year itemized"),
    ("LINE_SCHEDULE_AI_FULL_YEAR_SELF_EMPLOYMENT", "Schedule AI full year self-employment"),
    ("LINE_SCHEDULE_AI_FULL_YEAR_SOCIAL_WAGES", "Schedule AI full year social wages"),
    ("LINE_SCHEDULE_B_ORDINARY_DIVIDENDS", "Schedule B ordinary dividends"),
    ("LINE_SCHEDULE_B_SAVINGS_BOND_EXCLUSION", "Schedule B excludable savings bond interest"),
    ("LINE_SCHEDULE_B_TAXABLE_INTEREST", "Schedule B taxable interest"),
    ("LINE_SCHEDULE_B_TOTAL_INTEREST", "Schedule B total interest"),
    ("LINE_SCHEDULE_E_RENTAL_ROYALTY_TOTAL", "Schedule E rental royalty total"),
    (
        "LINE_K1_N_COMPONENT_N_SHORT_NET",
        "K-1 {0} component {1} short-term net capital gain or loss",
    ),
    (
        "LINE_K1_N_COMPONENT_N_SHORT_BASIS_ALLOWED",
        "K-1 {0} component {1} short-term capital loss allowed by basis",
    ),
    (
        "LINE_K1_N_COMPONENT_N_SHORT_BASIS_CARRY",
        "K-1 {0} component {1} short-term capital loss carried at basis",
    ),
    (
        "LINE_K1_N_COMPONENT_N_SHORT_RISK_ALLOWED",
        "K-1 {0} component {1} short-term capital loss allowed at risk",
    ),
    (
        "LINE_K1_N_COMPONENT_N_SHORT_RISK_CARRY",
        "K-1 {0} component {1} short-term capital loss carried at risk",
    ),
    ("LINE_K1_N_COMPONENT_N_LONG_NET", "K-1 {0} component {1} long-term net capital gain or loss"),
    (
        "LINE_K1_N_COMPONENT_N_LONG_BASIS_ALLOWED",
        "K-1 {0} component {1} long-term capital loss allowed by basis",
    ),
    (
        "LINE_K1_N_COMPONENT_N_LONG_BASIS_CARRY",
        "K-1 {0} component {1} long-term capital loss carried at basis",
    ),
    (
        "LINE_K1_N_COMPONENT_N_LONG_RISK_ALLOWED",
        "K-1 {0} component {1} long-term capital loss allowed at risk",
    ),
    (
        "LINE_K1_N_COMPONENT_N_LONG_RISK_CARRY",
        "K-1 {0} component {1} long-term capital loss carried at risk",
    ),
    (
        "LINE_AMT_K1_N_COMPONENT_N_SHORT_NET",
        "AMT K-1 {0} component {1} short-term net capital gain or loss",
    ),
    (
        "LINE_AMT_K1_N_COMPONENT_N_SHORT_BASIS_ALLOWED",
        "AMT K-1 {0} component {1} short-term capital loss allowed by basis",
    ),
    (
        "LINE_AMT_K1_N_COMPONENT_N_SHORT_BASIS_CARRY",
        "AMT K-1 {0} component {1} short-term capital loss carried at basis",
    ),
    (
        "LINE_AMT_K1_N_COMPONENT_N_SHORT_RISK_ALLOWED",
        "AMT K-1 {0} component {1} short-term capital loss allowed at risk",
    ),
    (
        "LINE_AMT_K1_N_COMPONENT_N_SHORT_RISK_CARRY",
        "AMT K-1 {0} component {1} short-term capital loss carried at risk",
    ),
    (
        "LINE_AMT_K1_N_COMPONENT_N_LONG_NET",
        "AMT K-1 {0} component {1} long-term net capital gain or loss",
    ),
    (
        "LINE_AMT_K1_N_COMPONENT_N_LONG_BASIS_ALLOWED",
        "AMT K-1 {0} component {1} long-term capital loss allowed by basis",
    ),
    (
        "LINE_AMT_K1_N_COMPONENT_N_LONG_BASIS_CARRY",
        "AMT K-1 {0} component {1} long-term capital loss carried at basis",
    ),
    (
        "LINE_AMT_K1_N_COMPONENT_N_LONG_RISK_ALLOWED",
        "AMT K-1 {0} component {1} long-term capital loss allowed at risk",
    ),
    (
        "LINE_AMT_K1_N_COMPONENT_N_LONG_RISK_CARRY",
        "AMT K-1 {0} component {1} long-term capital loss carried at risk",
    ),
    (
        "EXPLAIN_K1_COMPONENT_CAPITAL_NET",
        "This component’s portfolio capital income less its current and retained capital losses allowed after basis and at-risk limits. The signed source cents enter Schedule D once, outside ordinary passive income, self-employment and QBI.",
    ),
    (
        "EXPLAIN_K1_COMPONENT_CAPITAL_BASIS_ALLOWED",
        "Current and retained capital losses allowed by the shared whole-interest basis limit, before the actual source group’s at-risk limit.",
    ),
    (
        "EXPLAIN_K1_COMPONENT_CAPITAL_BASIS_CARRY",
        "Capital loss retained at basis for this component and holding term; it has not entered the at-risk limit.",
    ),
    (
        "EXPLAIN_K1_COMPONENT_CAPITAL_RISK_ALLOWED",
        "Capital loss allowed by this component’s actual at-risk group, including released prior at-risk losses. It enters the portfolio capital category, outside ordinary passive income.",
    ),
    (
        "EXPLAIN_K1_COMPONENT_CAPITAL_RISK_CARRY",
        "Capital loss retained by this component’s actual at-risk group. It has already passed basis and does not reduce basis again on release.",
    ),
    ("LINE_SCORP_N_SHORT_GAIN", "S corporation {0} short-term capital gain"),
    ("LINE_SCORP_N_LONG_GAIN", "S corporation {0} long-term capital gain"),
    (
        "EXPLAIN_K1_ACTIVITY_SHORT_GAIN",
        "Entity-only K-1 portfolio short-term capital gain, Schedule D line 5; it increases entity basis once, outside activity income.",
    ),
    (
        "EXPLAIN_K1_ACTIVITY_LONG_GAIN",
        "Entity-only K-1 portfolio long-term capital gain, Schedule D line 12; it increases entity basis once, outside activity income.",
    ),
    ("LINE_SCORP_N_BACKUP_WITHHOLDING", "S corporation {0} backup withholding"),
    ("LINE_SCORP_N_CASH_CHARITY", "S corporation {0} cash charity"),
    ("LINE_SCORP_N_EXEMPT_INTEREST", "S corporation {0} exempt interest"),
    ("LINE_SCORP_N_INTEREST", "S corporation {0} interest"),
    ("LINE_SCORP_N_DIVIDENDS", "S corporation {0} dividends"),
    ("LINE_SCORP_N_QUALIFIED_DIVIDENDS", "S corporation {0} qualified dividends"),
    ("LINE_AMT_SCORP_N_DIVIDENDS", "AMT S corporation {0} dividends"),
    ("LINE_AMT_SCORP_N_QUALIFIED_DIVIDENDS", "AMT S corporation {0} qualified dividends"),
    ("LINE_SCORP_N_DISTRIBUTION_LONG_AMT_GAIN", "S corporation {0} distribution long AMT gain"),
    ("LINE_SCORP_N_DISTRIBUTION_LONG_GAIN", "S corporation {0} distribution long gain"),
    ("LINE_SCORP_N_DISTRIBUTION_SHORT_AMT_GAIN", "S corporation {0} distribution short AMT gain"),
    ("LINE_SCORP_N_DISTRIBUTION_SHORT_GAIN", "S corporation {0} distribution short gain"),
    ("LINE_SCORP_N_LOAN_LONG_AMT_GAIN", "S corporation {0} loan long AMT gain"),
    ("LINE_SCORP_N_LOAN_LONG_GAIN", "S corporation {0} loan long gain"),
    ("LINE_SCORP_N_LOAN_SHORT_AMT_GAIN", "S corporation {0} loan short AMT gain"),
    ("LINE_SCORP_N_LOAN_SHORT_GAIN", "S corporation {0} loan short gain"),
    ("LINE_SCORP_N_SECTION1231", "S corporation {0} section 1231"),
    ("LINE_SCORP_N_SECTION179", "S corporation {0} section 179"),
    ("LINE_SCORP_N_SECTION179_CARRYFORWARD", "S corporation {0} section 179 carryforward"),
    ("LINE_SCORP_N_SECTION179_PRIOR_CARRYOVER", "S corporation {0} section 179 prior carryover"),
    ("LINE_SCORP_AMT_LOAN_GAIN_ADJUSTMENT", "S corporation AMT loan gain adjustment"),
    ("LINE_SCORP_ORDINARY_LOAN_GAIN", "S corporation ordinary loan gain"),
    ("LINE_SECTION1231_N_NEXT_LOOKBACK", "Section 1231 {0} next lookback"),
    ("LINE_SECTION1231_CURRENT_LOSS_HISTORY", "Section 1231 current loss history"),
    ("LINE_SECTION179_BUSINESS_INCOME_LIMIT", "Section 179 business income limit"),
    ("LINE_SECTION179_CARRYFORWARD", "Section 179 carryforward"),
    ("LINE_SECTION179_DEDUCTION", "Section 179 deduction"),
    ("LINE_SECTION179_DOLLAR_LIMIT", "Section 179 dollar limit"),
    ("LINE_SECTION179_ELIGIBLE_COST", "Section 179 eligible cost"),
    ("LINE_SECTION179_SHARED_DOLLAR_LIMIT", "Section 179 shared dollar limit"),
    ("LINE_SECTION179_SPOUSE_ELIGIBLE_COST", "Section 179 eligible cost, spouse"),
    ("LINE_SECTION_N_ORDINARY_LOSS", "Section {0} ordinary loss"),
    ("LINE_SELF_EMPLOYMENT_TAX", "Self-employment tax"),
    ("LINE_SENIOR_DEDUCTION", "Senior deduction"),
    ("LINE_SOCIAL_LUMP_SUM_TAXABLE_REDUCTION", "Social lump sum taxable reduction"),
    ("LINE_SOCIAL_SECURITY_NET_REPAYMENTS", "Social Security net repayments"),
    ("LINE_SPOUSE_HALF_SELF_EMPLOYMENT_TAX", "Spouse half self-employment tax"),
    ("LINE_SPOUSE_IRA_BASIS_CARRYFORWARD", "Spouse IRA basis carryforward"),
    ("LINE_SPOUSE_IRA_BASIS_RECOVERED", "Spouse IRA basis recovered"),
    ("LINE_SPOUSE_IRA_BASIS_TAXABLE", "Spouse IRA basis taxable"),
    ("LINE_SPOUSE_IRA_CURRENT_NONDEDUCTIBLE_BASIS", "Spouse IRA current nondeductible basis"),
    ("LINE_SPOUSE_IRA_INCOMING_PLAN_BASIS", "Spouse IRA incoming plan basis"),
    ("LINE_SPOUSE_IRA_OUTSTANDING_ROLLOVER", "Spouse IRA outstanding rollover"),
    ("LINE_SPOUSE_IRA_REPAYMENT_INCOME_REDUCTION", "Spouse IRA repayment income reduction"),
    ("LINE_SPOUSE_IRA_REPAYMENT_LATER_DEPOSITS", "Spouse IRA repayment later deposits"),
    ("LINE_SPOUSE_IRA_REPAYMENT_QUALIFIED_GROSS", "Spouse IRA repayment qualified gross"),
    ("LINE_SPOUSE_IRA_REPAYMENT_QUALIFIED_TAXABLE", "Spouse IRA repayment qualified taxable"),
    ("LINE_SPOUSE_IRA_REPAYMENT_TAXABLE_AFTER", "Spouse IRA repayment taxable after"),
    ("LINE_SPOUSE_IRA_REPAYMENT_YEAR_END", "Spouse IRA repayment year end"),
    ("LINE_SPOUSE_IRA_RESERVIST_BASIS", "Spouse IRA reservist basis"),
    ("LINE_SPOUSE_NONDEDUCTIBLE_IRA", "Spouse nondeductible IRA"),
    ("LINE_SPOUSE_SE_COMBINED_PROFIT", "Spouse SE combined profit"),
    ("LINE_SPOUSE_SE_CRP_PAYMENTS", "Spouse SE CRP payments (Conservation Reserve Program)"),
    ("LINE_SPOUSE_SE_FARM_PROFIT", "Spouse SE farm profit"),
    ("LINE_SPOUSE_SE_NET_EARNINGS", "Spouse SE net earnings"),
    ("LINE_SPOUSE_SE_NET_EARNINGS_ORDINARY", "Spouse SE net earnings ordinary"),
    ("LINE_SPOUSE_SE_OPTIONAL_EARNINGS", "Spouse SE optional method earnings"),
    ("LINE_SPOUSE_SE_NONFARM_PROFIT", "Spouse SE nonfarm profit"),
    ("LINE_SPOUSE_SELF_EMPLOYMENT_TAX", "Spouse self-employment tax"),
    ("LINE_STATE_REFUND_N_N_ALLOCATED", "State refund {0} {1} allocated"),
    ("LINE_STATE_REFUND_N_N_NET_INCOME_TAX_PAID", "State refund {0} {1} net income tax paid"),
    ("LINE_STATE_REFUND_N_N_TAXABLE", "State refund {0} {1} taxable"),
    ("LINE_STOCK_N_CAPITAL_BASIS_LOSS", "Stock {0} capital basis loss"),
    ("LINE_STOCK_N_ORDINARY_BEFORE_LIMIT", "Stock {0} ordinary before limit"),
    ("LINE_TAX_CREDIT_BOND_CARRY_N_N", "Tax credit bond carry {0} {1}"),
    ("LINE_TAX_CREDIT_BOND_CREDIT", "Tax credit bond credit"),
    ("LINE_TAX_CREDIT_BOND_USED_N", "Tax credit bond used {0}"),
    ("LINE_TAXABLE_ALIMONY", "Taxable alimony"),
    ("LINE_TAXABLE_CASUALTY_LIVING_INSURANCE", "Taxable casualty living insurance"),
    ("LINE_TAXABLE_GRANTS_1099G", "Taxable grants 1099-G"),
    ("LINE_TAXABLE_INCOME", "Taxable income"),
    ("LINE_TAXABLE_LTC_BENEFITS", "Taxable LTC benefits"),
    ("LINE_TAXABLE_HEALTH_LOAN_ASSISTANCE", "Taxable health-service loan assistance"),
    ("LINE_TAXABLE_PRIZES", "Taxable prizes"),
    ("LINE_TAXABLE_SCHOLARSHIPS", "Taxable scholarships"),
    ("LINE_FARM_N_CONSERVATION_AMT_DEDUCTION", "Farm {0} AMT conservation deduction"),
    ("LINE_FARM_N_CONSERVATION_AMT_CARRYFORWARD", "Farm {0} AMT conservation carryforward"),
    ("LINE_FORM_N_SPOUSE_N_EXCESS_REPAYMENT", "Form {0} {1} excess repayment, spouse"),
    ("LINE_FORM_N_SPOUSE_N_INSTALLMENT_N", "Form {0} {1} installment {2}, spouse"),
    ("LINE_FORM_N_SPOUSE_N_INSTALLMENT_FOLLOWING_YEAR", "Form {0} {1} installment 2027, spouse"),
    (
        "LINE_FORM_N_SPOUSE_N_INSTALLMENT_SECOND_FOLLOWING_YEAR",
        "Form {0} {1} installment 2028, spouse",
    ),
    ("LINE_FORM_N_SPOUSE_N_LINE_N", "Form {0} {1} line {2}, spouse"),
    ("LINE_SPOUSE_N_FORM_N_BOX10_N_DEPOSIT", "Spouse {0} Form {1} box 10 {2} deposit"),
    ("LINE_SPOUSE_N_FORM_N_BOX1_N_DEPOSIT", "Spouse {0} Form {1} box 1 {2} deposit"),
    ("LINE_SPOUSE_N_ROTH_FORM_N_BOX10", "Spouse {0} Roth Form {1} box 10"),
    ("LINE_SPOUSE_N_ROTH_SOURCE_CONTRIBUTION_BASIS", "Spouse {0} Roth source contribution basis"),
    ("LINE_SPOUSE_N_ROTH_TRANSFER_N_AMOUNT", "Spouse {0} Roth transfer {1} amount"),
    ("LINE_SPOUSE_IRA_TIMELY_RETURN_EARNINGS", "Spouse IRA timely return earnings"),
    ("LINE_SPOUSE_IRA_TIMELY_RETURN_GROSS", "Spouse IRA timely return gross"),
    ("LINE_SPOUSE_ROTH_IRA_TIMELY_RETURN_EARNINGS", "Spouse Roth IRA timely return earnings"),
    ("LINE_SPOUSE_ROTH_IRA_TIMELY_RETURN_GROSS", "Spouse Roth IRA timely return gross"),
    ("LINE_TAXABLE_SOCIAL_SECURITY", "Taxable Social Security"),
    ("LINE_TAXABLE_STATE_TAX_REFUND", "Taxable state tax refund"),
    ("LINE_TAXPAYER_N_FORM_N_BOX10_N_DEPOSIT", "Taxpayer {0} Form {1} box 10 {2} deposit"),
    ("LINE_TAXPAYER_N_FORM_N_BOX1_N_DEPOSIT", "Taxpayer {0} Form {1} box 1 {2} deposit"),
    ("LINE_TAXPAYER_N_ROTH_FORM_N_BOX10", "Taxpayer {0} Roth Form {1} box 10"),
    (
        "LINE_TAXPAYER_N_ROTH_SOURCE_CONTRIBUTION_BASIS",
        "Taxpayer {0} Roth source contribution basis",
    ),
    ("LINE_TAXPAYER_N_ROTH_TRANSFER_N_AMOUNT", "Taxpayer {0} Roth transfer {1} amount"),
    ("LINE_TAXPAYER_HALF_SELF_EMPLOYMENT_TAX", "Taxpayer half self-employment tax"),
    ("LINE_TAXPAYER_IRA_BASIS_CARRYFORWARD", "Taxpayer IRA basis carryforward"),
    ("LINE_TAXPAYER_IRA_BASIS_RECOVERED", "Taxpayer IRA basis recovered"),
    ("LINE_TAXPAYER_IRA_BASIS_TAXABLE", "Taxpayer IRA basis taxable"),
    ("LINE_TAXPAYER_IRA_CURRENT_NONDEDUCTIBLE_BASIS", "Taxpayer IRA current nondeductible basis"),
    ("LINE_TAXPAYER_IRA_INCOMING_PLAN_BASIS", "Taxpayer IRA incoming plan basis"),
    ("LINE_TAXPAYER_IRA_OUTSTANDING_ROLLOVER", "Taxpayer IRA outstanding rollover"),
    ("LINE_TAXPAYER_IRA_REPAYMENT_INCOME_REDUCTION", "Taxpayer IRA repayment income reduction"),
    ("LINE_TAXPAYER_IRA_REPAYMENT_LATER_DEPOSITS", "Taxpayer IRA repayment later deposits"),
    ("LINE_TAXPAYER_IRA_REPAYMENT_QUALIFIED_GROSS", "Taxpayer IRA repayment qualified gross"),
    ("LINE_TAXPAYER_IRA_REPAYMENT_QUALIFIED_TAXABLE", "Taxpayer IRA repayment qualified taxable"),
    ("LINE_TAXPAYER_IRA_REPAYMENT_TAXABLE_AFTER", "Taxpayer IRA repayment taxable after"),
    ("LINE_TAXPAYER_IRA_REPAYMENT_YEAR_END", "Taxpayer IRA repayment year end"),
    ("LINE_TAXPAYER_IRA_RESERVIST_BASIS", "Taxpayer IRA reservist basis"),
    ("LINE_TAXPAYER_IRA_TIMELY_RETURN_EARNINGS", "Taxpayer IRA timely return earnings"),
    ("LINE_TAXPAYER_IRA_TIMELY_RETURN_GROSS", "Taxpayer IRA timely return gross"),
    ("LINE_TAXPAYER_NONDEDUCTIBLE_IRA", "Taxpayer nondeductible IRA"),
    ("LINE_TAXPAYER_ROTH_IRA_TIMELY_RETURN_EARNINGS", "Taxpayer Roth IRA timely return earnings"),
    ("LINE_TAXPAYER_ROTH_IRA_TIMELY_RETURN_GROSS", "Taxpayer Roth IRA timely return gross"),
    ("LINE_TAXPAYER_SE_COMBINED_PROFIT", "Taxpayer SE combined profit"),
    ("LINE_TAXPAYER_SE_CRP_PAYMENTS", "Taxpayer SE CRP payments (Conservation Reserve Program)"),
    ("LINE_TAXPAYER_SE_FARM_PROFIT", "Taxpayer SE farm profit"),
    ("LINE_TAXPAYER_SE_NET_EARNINGS", "Taxpayer SE net earnings"),
    ("LINE_TAXPAYER_SE_NET_EARNINGS_ORDINARY", "Taxpayer SE net earnings ordinary"),
    ("LINE_TAXPAYER_SE_OPTIONAL_EARNINGS", "Taxpayer SE optional method earnings"),
    ("LINE_TAXPAYER_SE_NONFARM_PROFIT", "Taxpayer SE nonfarm profit"),
    ("LINE_TAXPAYER_SELF_EMPLOYMENT_TAX", "Taxpayer self-employment tax"),
    ("LINE_TIP_FICA_N_CARRYBACK_USED", "Tip FICA {0} carryback used"),
    ("LINE_TIP_FICA_N_CREDIT_ORIGIN", "Tip FICA {0} credit origin"),
    ("LINE_TIP_FICA_N_CREDIT_USED", "Tip FICA {0} credit used"),
    ("LINE_TIP_FICA_CREDIT_CARRYFORWARD_N", "Tip FICA credit carryforward {0}"),
    ("LINE_TIP_FICA_CURRENT_CREDIT", "Tip FICA current credit"),
    ("LINE_TIP_FICA_CURRENT_CREDIT_CARRYBACK_N", "Tip FICA current credit carryback {0}"),
    ("LINE_TIP_FICA_CURRENT_CREDIT_CARRYFORWARD_N", "Tip FICA current credit carryforward {0}"),
    ("LINE_TIP_FICA_CURRENT_CREDIT_USED", "Tip FICA current credit used"),
    ("LINE_TIPS_DEDUCTION", "Tips deduction"),
    ("LINE_TOTAL_TAX", "Total tax"),
    ("LINE_TRUMP_ABLE_ROLLOVER_N_GROSS", "Trump ABLE rollover {0} gross"),
    ("LINE_TRUMP_ACCOUNT_TAXPAYER_LINE57_EXCESS", "Trump account line 57 excess, taxpayer"),
    ("LINE_TRUMP_ACCOUNT_TAXPAYER_LINE59_RETURNED", "Trump account line 59 returned, taxpayer"),
    ("LINE_TRUMP_ACCOUNT_TAXPAYER_LINE60_REMAINING", "Trump account line 60 remaining, taxpayer"),
    ("LINE_TRUMP_ACCOUNT_TAXPAYER_LINE61_TAX", "Trump account line 61 tax, taxpayer"),
    ("LINE_TRUMP_ACCOUNT_TAXPAYER_LINE62_EARNINGS", "Trump account line 62 earnings, taxpayer"),
    ("LINE_TRUMP_ACCOUNT_TAXPAYER_LINE63_TAX", "Trump account line 63 tax, taxpayer"),
    ("LINE_TRUMP_DEATH_N_TAXABLE", "Trump death {0} taxable"),
    ("LINE_TRUMP_DEATH_N_VALUE", "Trump death {0} value"),
    ("LINE_TRUMP_DEATH_TAXABLE_INCOME", "Trump death taxable income"),
    ("LINE_UNDISTRIBUTED_GAIN_TAX_CREDIT", "Undistributed gain tax credit"),
    ("LINE_UNEMPLOYMENT_N_AGRICULTURE_PAYMENT", "Unemployment {0} agriculture payment"),
    ("LINE_UNEMPLOYMENT_N_FAMILY_LEAVE", "Unemployment {0} family leave"),
    ("LINE_UNEMPLOYMENT_N_GROSS", "Unemployment {0} gross"),
    ("LINE_UNEMPLOYMENT_N_REPAID", "Unemployment {0} repaid"),
    ("LINE_UNEMPLOYMENT_N_TAXABLE", "Unemployment {0} taxable"),
    ("LINE_UNEMPLOYMENT_N_TAXABLE_GRANTS", "Unemployment {0} taxable grants"),
    ("LINE_UNEMPLOYMENT_PRIOR_YEAR_REPAYMENTS", "Unemployment prior year repayments"),
    ("LINE_UNRECAPTURED_SECTION_N_GAIN", "Unrecaptured section {0} gain"),
    ("LINE_UNRECOVERED_ANNUITY_DEDUCTION", "Unrecovered annuity deduction"),
    ("LINE_VARIABLE_ANNUITY_N_ANNUAL_EXCLUSION", "Variable annuity {0} annual exclusion"),
    ("LINE_VARIABLE_ANNUITY_N_BASIS_RECOVERY", "Variable annuity {0} basis recovery"),
    ("LINE_VARIABLE_ANNUITY_N_EARLIER_RECOVERY", "Variable annuity {0} earlier recovery"),
    ("LINE_VARIABLE_ANNUITY_N_ELECTION", "Variable annuity {0} election"),
    ("LINE_VARIABLE_ANNUITY_N_REMAINING_BASIS", "Variable annuity {0} remaining basis"),
    ("LINE_VEHICLE_INTEREST_DEDUCTION", "Vehicle interest deduction"),
    ("LINE_W2_UNCOLLECTED_PAYROLL_TAX", "W-2 uncollected payroll tax"),
    ("LINE_WAGES", "Wages"),
    ("LINE_WILDFIRE_RELIEF_N_EXCLUDED", "Wildfire relief {0} excluded"),
    ("LINE_WORTHLESS_SECURITY_N_LOSS", "Worthless security {0} loss"),
    // Calculated return rows (row_label): form, printed line, then the label.
    ("ROW_CHILD_CREDIT_AGI", "{0} {1} — Adjusted gross income"),
    ("ROW_CHILD_CREDIT_FOREIGN_EXCLUSIONS", "{0} {1} — Foreign exclusions added back"),
    ("ROW_CHILD_CREDIT_EXCLUSIONS_TOTAL", "{0} {1} — Excluded income added back"),
    ("ROW_CHILD_CREDIT_MAGI", "{0} {1} — Modified adjusted gross income"),
    ("ROW_CHILD_CREDIT_CHILDREN", "{0} {1} — Qualifying children under 17 with the required SSN"),
    ("ROW_CHILD_CREDIT_CHILDREN_AMOUNT", "{0} {1} — Child tax credit before the phase-out"),
    ("ROW_CHILD_CREDIT_OTHER_DEPENDENTS", "{0} {1} — Other dependents"),
    (
        "ROW_CHILD_CREDIT_OTHER_DEPENDENTS_AMOUNT",
        "{0} {1} — Credit for other dependents before the phase-out",
    ),
    ("ROW_CHILD_CREDIT_BEFORE_PHASEOUT", "{0} {1} — Credits before the phase-out"),
    ("ROW_CHILD_CREDIT_PHASEOUT_THRESHOLD", "{0} {1} — Phase-out threshold"),
    ("ROW_CHILD_CREDIT_EXCESS_INCOME", "{0} {1} — Income over the threshold"),
    ("ROW_CHILD_CREDIT_PHASEOUT_REDUCTION", "{0} {1} — Phase-out reduction"),
    ("ROW_CHILD_CREDIT_AFTER_PHASEOUT", "{0} {1} — Credits after the phase-out"),
    ("ROW_CHILD_CREDIT_LIMIT", "{0} {1} — Credit Limit Worksheet A"),
    ("ROW_ADDITIONAL_CHILD_CREDIT_UNUSED", "{0} {1} — Credit not used against tax"),
    ("ROW_ADDITIONAL_CHILD_CREDIT_CHILDREN_LIMIT", "{0} {1} — Children times $1,700"),
    ("ROW_ADDITIONAL_CHILD_CREDIT_SMALLER", "{0} {1} — Smaller of lines 16a and 16b"),
    ("ROW_ADDITIONAL_CHILD_CREDIT_EARNED_INCOME", "{0} {1} — Earned income"),
    ("ROW_ADDITIONAL_CHILD_CREDIT_EARNED_OVER_FLOOR", "{0} {1} — Earned income over $2,500"),
    ("ROW_ADDITIONAL_CHILD_CREDIT_EARNED_SHARE", "{0} {1} — 15% of earned income over $2,500"),
    (
        "ROW_ADDITIONAL_CHILD_CREDIT_WITHHELD_TAXES",
        "{0} {1} — Social Security and Medicare tax withheld",
    ),
    (
        "ROW_ADDITIONAL_CHILD_CREDIT_OTHER_PAYROLL",
        "{0} {1} — Other payroll taxes and the half self-employment deduction",
    ),
    ("ROW_ADDITIONAL_CHILD_CREDIT_PAYROLL_TOTAL", "{0} {1} — Payroll taxes added"),
    (
        "ROW_ADDITIONAL_CHILD_CREDIT_EIC_OFFSET",
        "{0} {1} — Earned income credit and excess Social Security tax",
    ),
    ("ROW_ADDITIONAL_CHILD_CREDIT_PAYROLL_NET", "{0} {1} — Payroll tax alternative"),
    ("ROW_ADDITIONAL_CHILD_CREDIT_LARGER", "{0} {1} — Larger of lines 20 and 25"),
    ("ROW_SCHEDULE_EIC_CHILD_N_NAME", "{0} {1} — Child {2} name"),
    ("ROW_SCHEDULE_EIC_CHILD_N_SSN", "{0} {1} — Child {2} SSN"),
    ("ROW_SCHEDULE_EIC_CHILD_N_DIED", "{0} {1} — Child {2} born and died in 2026"),
    ("ROW_SCHEDULE_EIC_CHILD_N_BIRTH_YEAR", "{0} {1} — Child {2} year of birth"),
    ("ROW_SCHEDULE_EIC_CHILD_N_STUDENT", "{0} {1} — Child {2} full-time student under 24"),
    ("ROW_SCHEDULE_EIC_CHILD_N_DISABLED", "{0} {1} — Child {2} permanently and totally disabled"),
    ("ROW_SCHEDULE_EIC_CHILD_N_MONTHS", "{0} {1} — Child {2} months lived with you"),
    ("ROW_CARE_EXPENSES_ALLOWED", "{0} {1} — Qualified expenses, up to the limit"),
    ("ROW_CARE_EXPENSES_SMALLEST", "{0} {1} — Smallest of lines 3, 4 and 5"),
    ("ROW_CARE_AGI", "{0} {1} — Adjusted gross income"),
    ("ROW_CARE_PERCENT", "{0} {1} — Credit percentage (percent)"),
    ("ROW_CARE_CREDIT_BEFORE_PRIOR", "{0} {1} — Credit on 2026 expenses"),
    ("ROW_CARE_CREDIT_TOTAL", "{0} {1} — Credit including 2025 expenses paid in 2026"),
    ("ROW_CARE_CREDIT_LIMIT", "{0} {1} — Credit Limit Worksheet"),
    (
        "ROW_FORM8582_PART_IX_N_ITEMS_UNALLOWED",
        "{0} {1} — Activity {2}: other-income losses, unallowed",
    ),
    (
        "ROW_FORM8582_PART_IX_N_ITEMS_ALLOWED",
        "{0} {1} — Activity {2}: other-income losses, allowed",
    ),
    (
        "ROW_FORM8582_PART_IX_N_ORDINARY_UNALLOWED",
        "{0} {1} — Activity {2}: ordinary loss, unallowed",
    ),
    ("ROW_FORM8582_PART_IX_N_ORDINARY_ALLOWED", "{0} {1} — Activity {2}: ordinary loss, allowed"),
    (
        "ROW_FORM8582_PART_IX_N_SECTION1231_UNALLOWED",
        "{0} {1} — Activity {2}: section 1231 loss, unallowed",
    ),
    (
        "ROW_FORM8582_PART_IX_N_SECTION1231_ALLOWED",
        "{0} {1} — Activity {2}: section 1231 loss, allowed",
    ),
    (
        "ROW_FORM8582_PART_IX_N_SHORT_PROPERTY_ALLOWED",
        "{0} {1} — Activity {2}: short-held property loss, allowed",
    ),
    (
        "ROW_FORM8582_PART_IX_N_SHORT_PROPERTY_UNALLOWED",
        "{0} {1} — Activity {2}: short-held property loss, unallowed",
    ),
    ("ROW_CARE_EARNED_SPOUSE", "{0} {1} — Spouse’s earned income, or yours again"),
    ("ROW_CARE_EARNED_TAXPAYER", "{0} {1} — Your earned income"),
    ("ROW_QUALIFIED_NET_DISASTER_LOSS", "{0} {1} — Qualified disaster loss"),
    // Form 1040 rows and calculation explanations.
    (
        "EXPLAIN_0ESTATE_TRUST_N_PASSIVE_ACTIVITY_INCOME",
        "Schedule K-1 (Form 1041) boxes 6–8 passive income after box 9 deductions: Schedule E line 33 column (d), available on Form 8582 to absorb other passive losses, and net investment income. Not additional income.",
    ),
    (
        "LINE_PARTNERSHIP_N_GROUP_N_ACTIVITY_NET",
        "Partnership {0} group {1} ordinary result after passive limits",
    ),
    (
        "LINE_PARTNERSHIP_N_GROUP_N_PASSIVE_SUSPENDED",
        "Partnership {0} group {1} remaining passive loss",
    ),
    ("LINE_PARTNERSHIP_N_SCHEDULE_E", "Partnership {0} Schedule E ordinary business result"),
    (
        "LINE_AMT_PARTNERSHIP_N_GROUP_N_ACTIVITY_NET",
        "AMT partnership {0} group {1} ordinary result after passive limits",
    ),
    (
        "LINE_AMT_PARTNERSHIP_N_GROUP_N_PASSIVE_SUSPENDED",
        "AMT partnership {0} group {1} remaining passive loss",
    ),
    (
        "LINE_AMT_PARTNERSHIP_N_SCHEDULE_E",
        "AMT partnership {0} Schedule E ordinary business result",
    ),
    (
        "LINE_SCORP_N_GROUP_N_ACTIVITY_NET",
        "S corporation {0} group {1} ordinary result after passive limits",
    ),
    (
        "LINE_SCORP_N_GROUP_N_PASSIVE_SUSPENDED",
        "S corporation {0} group {1} remaining passive loss",
    ),
    ("LINE_SCORP_N_SCHEDULE_E", "S corporation {0} Schedule E ordinary business result"),
    (
        "LINE_AMT_SCORP_N_GROUP_N_ACTIVITY_NET",
        "AMT S corporation {0} group {1} ordinary result after passive limits",
    ),
    (
        "LINE_AMT_SCORP_N_GROUP_N_PASSIVE_SUSPENDED",
        "AMT S corporation {0} group {1} remaining passive loss",
    ),
    ("LINE_AMT_SCORP_N_SCHEDULE_E", "AMT S corporation {0} Schedule E ordinary business result"),
    (
        "EXPLAIN_SCORP_ACTIVITY_NET",
        "Ordinary S-corporation result after basis, at-risk and passive limits. Return-level limits and tax classifications remain separate.",
    ),
    (
        "EXPLAIN_SCORP_ACTIVITY_PASSIVE_SUSPENDED",
        "Ordinary S-corporation loss remaining after the shared passive limit; distinct from basis and at-risk carries. No rental special allowance applies to this business activity.",
    ),
    (
        "EXPLAIN_K1_ACTIVITY_ENTITY_BASIS",
        "Ending outside basis of the whole partnership interest after the entity's income, distributions and basis-limited losses. Each activity shares this one basis worksheet.",
    ),
    (
        "EXPLAIN_K1_ACTIVITY_BASIS_ALLOWED",
        "This activity's current and retained ordinary loss allowed by the whole-interest basis limit, before its at-risk and passive limits.",
    ),
    (
        "EXPLAIN_K1_ACTIVITY_BASIS_SUSPENDED",
        "Ordinary loss retained by the whole-interest basis limit for this source. Keep it separate from at-risk and passive carryforwards.",
    ),
    (
        "EXPLAIN_K1_ACTIVITY_RISK_ALLOWED",
        "Ordinary loss allowed from this source by its actual at-risk group, including released at-risk carry. Passive and return-level limits still apply.",
    ),
    (
        "EXPLAIN_K1_ACTIVITY_RISK_SUSPENDED",
        "Ordinary loss retained by this source's at-risk group. It has already passed basis and must not reduce basis again when released.",
    ),
    (
        "EXPLAIN_K1_ACTIVITY_PASSIVE_OPENING",
        "This source's retained ordinary loss that already passed basis and at-risk limits, before the passive group's current-year release or common passive allocation.",
    ),
    (
        "EXPLAIN_K1_ACTIVITY_BEFORE_PASSIVE",
        "Current ordinary result of this actual passive group after basis and at-risk limits, before subtracting its separate prior passive loss or applying the return's passive pool.",
    ),
    (
        "EXPLAIN_0PARTNERSHIP_N_ACTIVITY_NET",
        "Ordinary partnership result after basis, at-risk and passive limits, plus nonpassive guaranteed payments and any separately entered prior Form 4136 credit recognized on Schedule E. Return-level limits and tax classifications remain separate.",
    ),
    (
        "EXPLAIN_0PARTNERSHIP_N_PASSIVE_SUSPENDED",
        "Ordinary partnership loss remaining after the shared passive limit; distinct from basis and at-risk carries. No rental special allowance applies to this business activity.",
    ),
    (
        "EXPLAIN_0PERSONAL_RENTAL_N_N_ALLOWED_EXPENSES",
        "Current paid expenses and depreciation plus released prior deductions allowed on Schedule 1 line 24b. Do not deduct the opening balance again.",
    ),
    (
        "EXPLAIN_0PERSONAL_RENTAL_N_N_PRIOR_SUSPENDED_EXPENSES",
        "Opening suspended operating expenses and depreciation from this activity's retained tax worksheets; not an additional payment or a second reduction of asset basis.",
    ),
    (
        "EXPLAIN_0PERSONAL_RENTAL_N_N_SUSPENDED_LOSS",
        "Closing suspended operating expenses and depreciation for this activity. Zero means the entire available expense balance was allowed. Retain the separate regular-tax and AMT records.",
    ),
    (
        "EXPLAIN_0PERSONAL_RENTAL_PASSIVE_INCOME",
        "Personal-property rental profit available to offset passive losses, already reported on Schedule 1. Recharacterized nonpassive rent is excluded. This is not additional taxable income.",
    ),
    (
        "EXPLAIN_0RENTAL_INCOME",
        "Rental profit less currently allowed losses, including the phased-out active-participation allowance.",
    ),
    (
        "EXPLAIN_0RENTAL_MAGI",
        "Rental-loss MAGI excludes passive income/losses, taxable Social Security, IRA/student-loan deductions and the half-SE-tax deduction.",
    ),
    (
        "EXPLAIN_0RENTAL_N_DEPRECIATION",
        "AMT residential or nonresidential depreciation: 40-year ADS for pre-1999 buildings; otherwise 27.5 years for residential or 39 years for nonresidential real property. Mid-month convention; land excluded.",
    ),
    (
        "EXPLAIN_0RENTAL_N_N_HOME_DEPRECIATION_CARRYOVER",
        "Worksheet 5-1 line 7b: combined casualty and depreciation above the rent left after operating expenses. Retain its separate regular-tax and AMT amounts and its actual casualty character for the next worksheet.",
    ),
    (
        "EXPLAIN_0RENTAL_N_N_HOME_OPERATING_CARRYOVER",
        "Worksheet 5-1 line 7a: rental share of operating expenses above the rent left after interest, taxes and rental-only expenses. Enter it next year as this property's operating carryover.",
    ),
    ("EXPLAIN_0RENTAL_N_SUSPENDED_LOSS", "No passive loss remains for a property sold in full."),
    (
        "EXPLAIN_0RENTAL_N_SUSPENDED_LOSS_2",
        "Current and prior loss remaining after passive-activity limits; retain this property balance for a later return.",
    ),
    (
        "EXPLAIN_0RENTAL_SUSPENDED_LOSS",
        "Total current and prior passive loss remaining for a later year.",
    ),
    ("EXPLAIN_1040_10", "Calculated supported Schedule 1 adjustments."),
    ("EXPLAIN_1040_11A_11B", "Total income less adjustments."),
    ("EXPLAIN_1040_12E", "The selected supported deduction route."),
    ("EXPLAIN_1040_12F", "Eligible cash contributions when using the standard deduction."),
    (
        "EXPLAIN_1040_13A",
        "Supported Schedule 1-A senior, tips, overtime and vehicle-interest deductions.",
    ),
    ("EXPLAIN_1040_13B", "Calculated supported QBI deduction."),
    ("EXPLAIN_1040_14", "Sum of supported deductions."),
    ("EXPLAIN_1040_15", "Adjusted gross income less deductions, floored at zero."),
    (
        "EXPLAIN_1040_16",
        "2026 tax on current income plus any positive Form 8978 line 14, any Form 4972 tax and any recapture of an education credit under the 2025 Form 1040 line 16 instructions. Check line 16 box 2 for Form 4972 and box 3 for Form 8978 (identify it) or for the education credit recapture (enter ECR). Current-year ordinary-tax lookups below $100,000 use the September 2026 draft Tax Table; Schedule J and Form 8615 retain their own calculations.",
    ),
    (
        "EXPLAIN_1040_17",
        "Schedule 2 Part I: Form 6251 AMT, excess advance premium credit and transferred clean-vehicle credit repayments.",
    ),
    (
        "EXPLAIN_1040_18",
        "Regular income tax including positive Form 8978 adjustments, AMT, advance premium credit repayment and transferred clean-vehicle credit repayment.",
    ),
    ("EXPLAIN_1040_19", "Nonrefundable portion after eligibility, phaseouts and credit ordering."),
    ("EXPLAIN_1040_1A", "W-2 box 1 wages after any supported Form 8958 community allocation."),
    (
        "EXPLAIN_1040_1C",
        "Additional cash and noncash tip income; allocated estimates are replaced by adequate actual records.",
    ),
    ("EXPLAIN_1040_1E", "Employee benefits not excluded and not already included in W-2 box 1."),
    (
        "EXPLAIN_1040_1F",
        "Net Form 8839 employer-benefit wage adjustment, including a prior-year foreign-benefit or qualified special-needs exclusion; W-2 box 1 remains unchanged.",
    ),
    (
        "EXPLAIN_1040_1G",
        "Employee compensation omitted from W-2; actual payroll coverage determines its employee payroll tax. Not Schedule C receipts.",
    ),
    (
        "EXPLAIN_1040_1H",
        "Foreign employee pay, taxable employer disability pensions before minimum retirement age, military moving reimbursements above the move's cost, and elective deferrals above the annual limit.",
    ),
    (
        "EXPLAIN_1040_1Z",
        "W-2 wages, additional employee tips and Form 8919 wages, taxable disability pensions, dependent-care/adoption benefits and foreign employee pay before exclusions.",
    ),
    (
        "EXPLAIN_1040_20",
        "Supported foreign tax, education, care, elderly, adoption, minimum-tax, mortgage-interest, DC homebuyer, residential-energy, employer tip FICA carryforward, personal refueling and Form 8978 credits, and Saver's Credits.",
    ),
    ("EXPLAIN_1040_21", "Child credit plus supported Schedule 3 credits."),
    ("EXPLAIN_1040_22", "Income tax remaining after nonrefundable credits."),
    (
        "EXPLAIN_1040_23",
        "Self-employment, Additional Medicare, net investment income, retirement and HSA/MSA withdrawal taxes, Trump-account excess and returned-income taxes, plus Form 8828 mortgage-subsidy and prior education-credit recapture, after any allowed negative Form 8978 Schedule 2 line 13z adjustment.",
    ),
    ("EXPLAIN_1040_24A", "Income tax after credits plus supported additional taxes."),
    ("EXPLAIN_1040_24B", "Form 1062 line 15; first installment due at the unextended deadline."),
    (
        "EXPLAIN_1040_24C",
        "Line 24a plus 24b; the full elected liability is separately offset through Schedule 3, not treated as an actual payment.",
    ),
    ("EXPLAIN_1040_25A", "Form W-2 box 2 federal income tax withholding."),
    (
        "EXPLAIN_1040_25B",
        "Federal income tax withholding attributed to Form 1099 payer records; verify the source document for statements that do not identify a form.",
    ),
    (
        "EXPLAIN_1040_25C",
        "Withholding on Forms W-2G and Schedule K-1, other non-1099 payer records, and Form 8959 Additional Medicare withholding.",
    ),
    (
        "EXPLAIN_1040_25D",
        "Sum of Form 1040 lines 25a through 25c after each source line is rounded.",
    ),
    (
        "EXPLAIN_1040_26",
        "Entered federal estimated payments plus a beneficiary's Form 1041 K-1 box 13 code A allocation from timely Form 1041-T.",
    ),
    (
        "EXPLAIN_1040_27A",
        "Uses the draft 2026 IRS EIC Table and the earned-income/AGI worksheet sequence; final-table reconciliation remains required.",
    ),
    ("EXPLAIN_1040_28", "Calculated refundable child credit."),
    ("EXPLAIN_1040_29", "Calculated refundable American Opportunity credit."),
    (
        "EXPLAIN_1040_2A",
        "Tax-exempt interest documents; excluded from total income but used by applicable worksheets.",
    ),
    ("EXPLAIN_1040_2B", "Taxable interest documents."),
    (
        "EXPLAIN_1040_30",
        "Current-year refundable adoption credit; prior nonrefundable carries are excluded.",
    ),
    (
        "EXPLAIN_1040_31",
        "Extension payment, excess Social Security withholding, net premium tax credit, section 1341 repayment credit, Form 2439 and Form 4136 tax-paid credits, and the separate Form 1062 deferral offset.",
    ),
    ("EXPLAIN_1040_32A", "Sum of Form 1040 lines 27a, 28, 29, 30 and 31."),
    (
        "EXPLAIN_1040_32C",
        "Provisional line 32a amount: the draft Schedule 3-A reduction has not been applied; review the final rules before filing.",
    ),
    (
        "EXPLAIN_1040_33",
        "Payments, refundable credits and any Form 1062 deferral offset. The offset is not cash paid. Proposed Schedule 3-A reduction is not applied; read Estimate notes.",
    ),
    (
        "EXPLAIN_1040_34",
        "Before the estimated tax penalty on line 38, offsets, interest or an election to apply the overpayment to next year.",
    ),
    (
        "EXPLAIN_1040_34_2",
        "Before penalties, offsets, interest or an election to apply the overpayment to next year.",
    ),
    (
        "EXPLAIN_1040_35A",
        "Available overpayment after line 38, less the reported line 36 applied credit.",
    ),
    (
        "EXPLAIN_1040_36",
        "Planned applied credit, rounded once to whole dollars; the cash refund is the exact remainder. This draft is not a filed election or an actual payment.",
    ),
    ("EXPLAIN_1040_37", "Before penalties and interest."),
    (
        "EXPLAIN_1040_37_2",
        "Balance due plus the line 38 penalty, less any overpayment the penalty absorbs; before interest.",
    ),
    (
        "EXPLAIN_1040_38",
        "Form 2210 line 19 with the Schedule AI installments (box C), rounded to whole dollars.",
    ),
    ("EXPLAIN_1040_38_2", "Form 2210 line 19 by the regular method, rounded to whole dollars."),
    ("EXPLAIN_1040_38_3", "Form 2210-F line 16, rounded to whole dollars."),
    ("EXPLAIN_1040_3A", "Included in ordinary dividends; used for preferential tax rates."),
    ("EXPLAIN_1040_3B", "Total ordinary dividends, including qualified dividends once."),
    ("EXPLAIN_1040_4A", "IRA-category 1099-R box 1 amounts."),
    ("EXPLAIN_1040_4B", "IRA-category supported 1099-R box 2a amounts."),
    (
        "EXPLAIN_1040_5A",
        "Pension-category 1099-R box 1 amounts; a lump sum taxed on Form 4972 adds nothing, or only its ordinary part with the 20% capital gain rate alone.",
    ),
    (
        "EXPLAIN_1040_5B",
        "Pension-category payer amounts or calculated Simplified Method taxable benefits.",
    ),
    (
        "EXPLAIN_1040_6A",
        "Combined SSA-1099 and equivalent RRB-1099 box 5 net benefits, floored at zero after combining the return.",
    ),
    ("EXPLAIN_1040_6B", "Calculated using the Social Security worksheet and other return income."),
    (
        "EXPLAIN_1040_7A",
        "Net investment sales and capital-gain distributions after the supported annual loss limit.",
    ),
    (
        "EXPLAIN_1040_8",
        "Schedule 1 ordinary section 1231 income/loss, ISO compensation not in W-2 wages, prior NOL deduction, state income tax refund, ordinary shareholder-loan repayment gain, section 1244 ordinary stock loss, business income/loss, unemployment, gambling, jury pay, scholarships, prizes, taxable temporary-living-expense insurance, personal debt cancellation, taxable LTC benefits, hobby sales, alimony, HSA/MSA withdrawals and inherited values, rental and royalty income, foreign exclusion and excess-business-loss adjustment.",
    ),
    ("EXPLAIN_1040_9", "Taxable income categories before adjustments."),
    ("EXPLAIN_19A", "Alimony paid."),
    (
        "EXPLAIN_4A",
        "Schedule 1 lines 3, 5 and 6: business, rental, royalty, partnership, S corporation and farm income or loss.",
    ),
    (
        "EXPLAIN_4B",
        "Removes the income or loss of nonpassive trades or businesses, including a real estate professional's rentals under the 500-hour safe harbor; financial trading and passive activities stay in.",
    ),
    ("EXPLAIN_4C", "Line 4a combined with line 4b."),
    (
        "EXPLAIN_5A",
        "Form 1040 line 7 capital gain or loss plus Schedule 1 line 4 other gains or losses.",
    ),
    (
        "EXPLAIN_5B",
        "Gains and losses of nonpassive business property, deferred employer-stock appreciation and the excluded-loss carryforward difference.",
    ),
    ("EXPLAIN_5D", "Lines 5a through 5c combined."),
    (
        "EXPLAIN_SCHEDULE_1_1",
        "Taxable refunds, credits or offsets of state and local income taxes.",
    ),
    (
        "EXPLAIN_SCHEDULE_1_2A",
        "Alimony received under a pre-2019 instrument, and alimony recaptured from the payer in the third year.",
    ),
    (
        "EXPLAIN_SCHEDULE_1_4",
        "Other gains or losses: the ordinary part of property sales (Form 4797), S corporation loan-basis gains, and stock losses that are ordinary.",
    ),
    (
        "EXPLAIN_SCHEDULE_1_7",
        "Unemployment compensation received, less the same year's repayments.",
    ),
    ("EXPLAIN_SCHEDULE_1_8A", "Net operating loss deduction, shown as a negative amount."),
    ("EXPLAIN_SCHEDULE_1_8C", "Canceled debt included in income."),
    (
        "EXPLAIN_SCHEDULE_1_8D",
        "Foreign earned income exclusion from Form 2555, shown as a negative amount.",
    ),
    (
        "EXPLAIN_SCHEDULE_1_8E",
        "Income from Form 8853: Archer and Medicare Advantage MSA distributions and taxable long-term care benefits.",
    ),
    (
        "EXPLAIN_SCHEDULE_1_8F",
        "Income from Form 8889: health savings account amounts included in income.",
    ),
    ("EXPLAIN_SCHEDULE_1_8I", "Prizes and awards."),
    (
        "EXPLAIN_SCHEDULE_1_8K",
        "Stock options: compensation from incentive stock option and employee stock purchase plan sales that no W-2 reports.",
    ),
    (
        "EXPLAIN_SCHEDULE_1_8L",
        "Income from renting personal property for profit when the rental is not a business.",
    ),
    ("EXPLAIN_SCHEDULE_1_8P", "Section 461(l) excess business loss added back to income."),
    ("EXPLAIN_SCHEDULE_1_8Q", "Taxable distributions from an ABLE account."),
    ("EXPLAIN_SCHEDULE_1_9", "Total other income: lines 8a through 8z."),
    ("EXPLAIN_SCHEDULE_1_23", "Archer MSA deduction (Form 8853)."),
    (
        "EXPLAIN_SCHEDULE_1_24B",
        "Expenses of renting personal property for profit, matching the income on line 8l.",
    ),
    ("EXPLAIN_SCHEDULE_1_24F", "Contributions to a section 501(c)(18)(D) pension plan."),
    ("EXPLAIN_SCHEDULE_1_24J", "Housing deduction from Form 2555."),
    (
        "EXPLAIN_SCHEDULE_1_24K",
        "Excess deductions of section 67(e) expenses from Schedule K-1 (Form 1041).",
    ),
    ("EXPLAIN_SCHEDULE_1_25", "Total other adjustments: lines 24a through 24z."),
    (
        "EXPLAIN_SCHEDULE_2_1A",
        "Excess advance payments of the premium tax credit that you repay (Form 8962).",
    ),
    (
        "EXPLAIN_SCHEDULE_2_1B",
        "Repayment of the new clean vehicle credit (Form 8936 Schedule A Part II).",
    ),
    ("EXPLAIN_SCHEDULE_2_1C", "Repayment of the previously owned clean vehicle credit."),
    ("EXPLAIN_SCHEDULE_2_1Z", "Lines 1a through 1y added."),
    ("EXPLAIN_SCHEDULE_2_2", "Alternative minimum tax (Form 6251)."),
    ("EXPLAIN_SCHEDULE_2_3", "Lines 1z and 2 added: Form 1040 line 17."),
    (
        "EXPLAIN_SCHEDULE_2_5",
        "Additional tax on early distributions and other tax-favored account distributions (Form 5329 Parts I and II).",
    ),
    ("EXPLAIN_SCHEDULE_2_13B", "Recapture of the federal mortgage subsidy (Form 8828)."),
    (
        "EXPLAIN_SCHEDULE_2_13C",
        "Additional tax on health savings account distributions (Form 8889).",
    ),
    (
        "EXPLAIN_SCHEDULE_2_13D",
        "Additional tax when you fail the health savings account testing period (Form 8889).",
    ),
    ("EXPLAIN_SCHEDULE_2_13E", "Additional tax on Archer MSA distributions (Form 8853)."),
    (
        "EXPLAIN_SCHEDULE_2_13F",
        "Additional tax on Medicare Advantage MSA distributions (Form 8853).",
    ),
    ("LINE_SCHEDULE_N_GOLDEN_PARACHUTE_TAX", "Schedule {0} golden parachute excise tax"),
    ("LINE_MINIMUM_TAX_HISTORY_N_LINE_N", "Prior return {0} Form 8801 line {1}"),
    (
        "LINE_MINIMUM_TAX_HISTORY_N_FOREIGN_N_LINE_N",
        "Prior return {0} foreign category {1}, line {2}",
    ),
    (
        "LINE_MINIMUM_TAX_HISTORY_N_FOREIGN_TOTAL_LINE_N",
        "Prior return {0} exclusion-only foreign credit, line {1}",
    ),
    ("LINE_MINIMUM_TAX_CURRENT_REGULAR_LIABILITY", "Current regular tax after preceding credits"),
    ("LINE_MINIMUM_TAX_CURRENT_TENTATIVE", "Current tentative minimum tax"),
    (
        "LINE_MINIMUM_TAX_HISTORY_N_FOREIGN_EARNED_TAXABLE",
        "Prior return {0} foreign-earned-income worksheet line 1 taxable income",
    ),
    (
        "LINE_MINIMUM_TAX_HISTORY_N_FOREIGN_EARNED_EXCLUDED",
        "Prior return {0} foreign-earned-income worksheet line 2a exclusions and housing deduction",
    ),
    (
        "LINE_MINIMUM_TAX_HISTORY_N_FOREIGN_EARNED_RELATED",
        "Prior return {0} foreign-earned-income worksheet line 2b related unallowed deductions",
    ),
    (
        "LINE_MINIMUM_TAX_HISTORY_N_FOREIGN_EARNED_NET_EXCLUDED",
        "Prior return {0} foreign-earned-income worksheet line 2c net excluded income",
    ),
    (
        "LINE_MINIMUM_TAX_HISTORY_N_FOREIGN_EARNED_STACKED",
        "Prior return {0} foreign-earned-income worksheet line 3 stacked income",
    ),
    (
        "LINE_MINIMUM_TAX_HISTORY_N_FOREIGN_EARNED_STACKED_TAX",
        "Prior return {0} foreign-earned-income worksheet line 4 stacked tax",
    ),
    (
        "LINE_MINIMUM_TAX_HISTORY_N_FOREIGN_EARNED_EXCLUDED_TAX",
        "Prior return {0} foreign-earned-income worksheet line 5 tax on excluded income",
    ),
    (
        "LINE_MINIMUM_TAX_HISTORY_N_FOREIGN_EARNED_TAX",
        "Prior return {0} foreign-earned-income worksheet line 6 exclusion-only tax",
    ),
    (
        "EXPLAIN_MTC_RETAINED_WORKSHEET",
        "Retained prior-year exclusion-only worksheet. The 2026 credit uses 2025 source rules; separately filed histories keep their own prior status. Amounts do not use current-year rates or reconstruct an unavailable historical source.",
    ),
    (
        "EXPLAIN_MTC_CURRENT_LIMIT",
        "One current regular-tax limit after preceding credits, less current tentative minimum tax. The minimum-tax credit does not offset current AMT or section 4999 excise.",
    ),
    (
        "EXPLAIN_SCHEDULE_2_13K",
        "W-2 box 12 code K tax plus 20% of Form 1099-NEC box 3 excess golden parachute payments. These amounts do not add income or withholding again.",
    ),
    ("EXPLAIN_SCHEDULE_2_14", "Lines 13a through 13z added."),
    ("EXPLAIN_SCHEDULE_2_15", "Lines 4 through 11 and 14 added."),
    (
        "EXPLAIN_SCHEDULE_2_16A",
        "Social security and Medicare tax on tips you did not report to your employer (Form 4137).",
    ),
    (
        "EXPLAIN_SCHEDULE_2_16B",
        "Uncollected social security and Medicare tax on wages (Form 8919).",
    ),
    ("EXPLAIN_SCHEDULE_2_16C", "Lines 16a and 16b added."),
    ("EXPLAIN_SCHEDULE_2_17A", "Household employment taxes (Schedule H)."),
    (
        "EXPLAIN_SCHEDULE_2_17C",
        "Uncollected social security and Medicare or railroad retirement tax on tips or group-term life insurance (W-2 box 12 codes A, B, M and N).",
    ),
    ("EXPLAIN_SCHEDULE_2_17D", "Lines 17a through 17c added."),
    (
        "EXPLAIN_SCHEDULE_2_18",
        "Additional tax on excess contributions and accumulations (Form 5329 Parts III through IX).",
    ),
    ("EXPLAIN_SCHEDULE_2_20", "Lines 16c, 17d, 18 and 19c added."),
    ("EXPLAIN_SCHEDULE_2_21", "Lines 15 and 20 added: Form 1040 line 23."),
    ("EXPLAIN_SCHEDULE_3_1", "Foreign tax credit (Form 1116)."),
    ("EXPLAIN_SCHEDULE_3_2", "Credit for child and dependent care expenses (Form 2441 line 11)."),
    ("EXPLAIN_SCHEDULE_3_3", "Education credits (Form 8863 line 19)."),
    ("EXPLAIN_SCHEDULE_3_4", "Retirement savings contributions credit (Form 8880)."),
    ("EXPLAIN_SCHEDULE_3_5A", "Residential clean energy credit (Form 5695)."),
    ("EXPLAIN_SCHEDULE_3_6A", "General business credit (Form 3800)."),
    ("EXPLAIN_SCHEDULE_3_6B", "Credit for prior year minimum tax (Form 8801)."),
    ("EXPLAIN_SCHEDULE_3_6C", "Nonrefundable adoption credit (Form 8839)."),
    ("EXPLAIN_SCHEDULE_3_6D", "Credit for the elderly or the disabled (Schedule R)."),
    ("EXPLAIN_SCHEDULE_3_6F", "New clean vehicle credit (Form 8936)."),
    ("EXPLAIN_SCHEDULE_3_6G", "Mortgage interest credit (Form 8396)."),
    ("EXPLAIN_SCHEDULE_3_6H", "District of Columbia first-time homebuyer credit (Form 8859)."),
    ("EXPLAIN_SCHEDULE_3_6I", "Qualified electric vehicle credit (Form 8834)."),
    ("EXPLAIN_SCHEDULE_3_6J", "Alternative fuel vehicle refueling property credit (Form 8911)."),
    ("EXPLAIN_SCHEDULE_3_6K", "Credit to holders of tax credit bonds (Form 8912)."),
    ("EXPLAIN_SCHEDULE_3_6M", "Previously owned clean vehicle credit (Form 8936)."),
    ("EXPLAIN_SCHEDULE_3_7", "Lines 6a through 6z added."),
    ("EXPLAIN_SCHEDULE_3_8", "Lines 1 through 4, 5a and 7 added: Form 1040 line 20."),
    ("EXPLAIN_SCHEDULE_3_9", "Net premium tax credit (Form 8962 line 26)."),
    ("EXPLAIN_SCHEDULE_3_10", "Amount paid with a request for an extension to file."),
    (
        "EXPLAIN_SCHEDULE_3_11",
        "Excess social security and tier 1 railroad retirement tax withheld.",
    ),
    ("EXPLAIN_SCHEDULE_3_12", "Credit for federal tax on fuels (Form 4136)."),
    (
        "EXPLAIN_SCHEDULE_3_13B",
        "Credit for repayment of amounts included in income from earlier years.",
    ),
    (
        "EXPLAIN_SCHEDULE_3_13E",
        "Farm income tax deferral installment payment applied (Form 1062 line 14).",
    ),
    ("EXPLAIN_SCHEDULE_3_14", "Lines 13a through 13z added."),
    ("EXPLAIN_SCHEDULE_3_15", "Lines 9 through 12 and 14 added: Form 1040 line 31."),
    (
        "EXPLAIN_8Z",
        "Other income: taxable grants, paid family leave, temporary living-expense insurance above the increase, disaster or wildfire relief not confirmed as excluded, a prior-year mortgage interest refund, and the taxable value of a Trump account at death.",
    ),
    (
        "EXPLAIN_9A",
        "Schedule A line 9 investment interest, after the aggregate itemized-deduction cap.",
    ),
    (
        "EXPLAIN_9B",
        "State and local income tax allocated to investment income and real property tax on investment property, after the SALT and aggregate caps.",
    ),
    ("EXPLAIN_9C", "Miscellaneous investment expenses are not deductible."),
    ("EXPLAIN_9D", "Lines 9a through 9c."),
    (
        "EXPLAIN_ABLE_N_ANNUAL_TAXABLE",
        "This ABLE account's taxable annual earnings after its own Form 1099-QA earnings ratio, rollover treatment and nonduplicated qualified-expense allocation. Other accounts are calculated separately before Form 1040 and Form 5329 totals are added.",
    ),
    (
        "EXPLAIN_ABLE_N_FOLLOWING_EXPENSES",
        "Qualified disability expenses paid in the first 60 days of 2027 elected for 2026 under section 1.529A-3(a)(2); exclude these expenses from the 2027 calculation.",
    ),
    (
        "EXPLAIN_ABLE_N_N",
        "2026 ABLE base limit is $20,000. Only qualifying beneficiary-funded cash can use the additional compensation/prior-year poverty allowance. Form 5329 Part VIII taxes current excess at 6%, capped by year-end value; no prior-excess carry-in or income deduction.",
    ),
    (
        "EXPLAIN_ABLE_N_N_2",
        "Qualified rollover amount and its source-payment earnings are removed before the annual disability-expense ratio. The rollover is not a disability expense and is not entered twice.",
    ),
    (
        "EXPLAIN_ACQUISITION_N_GAIN",
        "Form 8949 sale of property a lender acquired, at its amount realized. A personal-use loss is disallowed.",
    ),
    (
        "EXPLAIN_ADDITIONAL_CHILD_CREDIT",
        "Unused credit subject to child count, earned-income and three-child payroll alternative limits.",
    ),
    (
        "EXPLAIN_CHILD_CREDIT_AGI",
        "Form 1040 line 11b, your adjusted gross income: where the child tax credit and credit for other dependents phase-out starts counting.",
    ),
    (
        "EXPLAIN_CHILD_CREDIT_FOREIGN_EXCLUSIONS",
        "Form 2555 lines 45 and 50, the foreign earned income and housing exclusions, added back to income for the phase-out. Puerto Rico and American Samoa exclusions (lines 2a and 2c) are not modeled.",
    ),
    (
        "EXPLAIN_CHILD_CREDIT_EXCLUSIONS_TOTAL",
        "Lines 2a through 2c added; only the Form 2555 amount on line 2b is modeled.",
    ),
    ("EXPLAIN_CHILD_CREDIT_MAGI", "Line 1 plus line 2d: the income the phase-out tests."),
    (
        "EXPLAIN_CHILD_CREDIT_CHILDREN",
        "Qualifying children under age 17 at the end of 2026 who have the required Social Security number; a child attains 17 on the birthday.",
    ),
    ("EXPLAIN_CHILD_CREDIT_CHILDREN_AMOUNT", "$2,200 for each qualifying child on line 4."),
    (
        "EXPLAIN_CHILD_CREDIT_OTHER_DEPENDENTS",
        "Other dependents, including qualifying children who are not under 17 or who lack the required SSN; never yourself, your spouse or anyone who is not a U.S. citizen, national or resident alien.",
    ),
    ("EXPLAIN_CHILD_CREDIT_OTHER_DEPENDENTS_AMOUNT", "$500 for each other dependent on line 6."),
    ("EXPLAIN_CHILD_CREDIT_BEFORE_PHASEOUT", "Lines 5 and 7 added."),
    (
        "EXPLAIN_CHILD_CREDIT_PHASEOUT_THRESHOLD",
        "$400,000 on a joint return and $200,000 otherwise.",
    ),
    (
        "EXPLAIN_CHILD_CREDIT_EXCESS_INCOME",
        "Modified adjusted gross income over the threshold, rounded up to the next multiple of $1,000.",
    ),
    (
        "EXPLAIN_CHILD_CREDIT_PHASEOUT_REDUCTION",
        "5% of line 10: $50 for each $1,000 over the threshold.",
    ),
    (
        "EXPLAIN_CHILD_CREDIT_AFTER_PHASEOUT",
        "Line 8 less line 11. If it is zero you cannot take the child tax credit, the credit for other dependents or the additional child tax credit.",
    ),
    (
        "EXPLAIN_CHILD_CREDIT_LIMIT",
        "Your tax after the credits that come before this one (Credit Limit Worksheet A line 13): the child tax credit and credit for other dependents cannot exceed it.",
    ),
    (
        "EXPLAIN_ADDITIONAL_CHILD_CREDIT_UNUSED",
        "Line 12 less line 14: the part that did not reduce your tax and can become the additional child tax credit.",
    ),
    (
        "EXPLAIN_ADDITIONAL_CHILD_CREDIT_CHILDREN_LIMIT",
        "$1,700 for each qualifying child counted on line 4.",
    ),
    (
        "EXPLAIN_ADDITIONAL_CHILD_CREDIT_SMALLER",
        "The most the additional child tax credit can be before the earned-income test.",
    ),
    (
        "EXPLAIN_ADDITIONAL_CHILD_CREDIT_EARNED_INCOME",
        "Your earned income, from the Earned Income Worksheet (wages, tips, net self-employment earnings, and the other items the schedule lists).",
    ),
    (
        "EXPLAIN_ADDITIONAL_CHILD_CREDIT_EARNED_OVER_FLOOR",
        "Earned income above $2,500; blank when it is $2,500 or less.",
    ),
    ("EXPLAIN_ADDITIONAL_CHILD_CREDIT_EARNED_SHARE", "15% of line 19."),
    (
        "EXPLAIN_ADDITIONAL_CHILD_CREDIT_WITHHELD_TAXES",
        "Part II-B, three or more children: Social Security and Medicare tax withheld (W-2 boxes 4 and 6), adjusted for the Additional Medicare Tax.",
    ),
    (
        "EXPLAIN_ADDITIONAL_CHILD_CREDIT_OTHER_PAYROLL",
        "Schedule 1 line 15 (half of self-employment tax) plus Schedule 2 lines 16c and 17c.",
    ),
    ("EXPLAIN_ADDITIONAL_CHILD_CREDIT_PAYROLL_TOTAL", "Lines 21 and 22 added."),
    (
        "EXPLAIN_ADDITIONAL_CHILD_CREDIT_EIC_OFFSET",
        "Form 1040 line 27a plus Schedule 3 line 11, which reduce the payroll-tax alternative.",
    ),
    ("EXPLAIN_ADDITIONAL_CHILD_CREDIT_PAYROLL_NET", "Line 23 less line 24, not below zero."),
    (
        "EXPLAIN_ADDITIONAL_CHILD_CREDIT_LARGER",
        "The credit is the smaller of line 17 and this amount.",
    ),
    ("EXPLAIN_SCHEDULE_EIC_CHILD_N_NAME", "Child’s name, from the people page: {0}."),
    (
        "EXPLAIN_SCHEDULE_EIC_CHILD_N_SSN",
        "{0} has a valid Social Security number to enter on line 2.",
    ),
    (
        "EXPLAIN_SCHEDULE_EIC_CHILD_N_DIED",
        "{0} was born and died in 2026 and had no SSN: enter “Died” on line 2 and attach the birth certificate, death certificate or hospital records showing a live birth.",
    ),
    ("EXPLAIN_SCHEDULE_EIC_CHILD_N_BIRTH_YEAR", "The year {0} was born."),
    (
        "EXPLAIN_SCHEDULE_EIC_CHILD_N_STUDENT",
        "{0} was under 24 at the end of 2026, a full-time student and younger than you: line 4a is Yes.",
    ),
    (
        "EXPLAIN_SCHEDULE_EIC_CHILD_N_DISABLED",
        "{0} was permanently and totally disabled during 2026: line 4b is Yes.",
    ),
    (
        "EXPLAIN_SCHEDULE_EIC_CHILD_N_MONTHS",
        "Months {0} lived with you in the United States during 2026: 7 for more than half the year but under 7 months, 12 for a child born or who died in 2026 whose home was yours.",
    ),
    (
        "EXPLAIN_CARE_EXPENSES_ALLOWED",
        "Line 2 column (d) expenses added, limited to $3,000 for one qualifying person or $6,000 for two or more (less any excluded employer benefits).",
    ),
    (
        "EXPLAIN_CARE_EXPENSES_SMALLEST",
        "The smallest of the allowed expenses and each spouse’s earned income.",
    ),
    ("EXPLAIN_CARE_AGI", "Form 1040 line 11a, which sets the credit percentage."),
    (
        "EXPLAIN_CARE_PERCENT",
        "The percentage from the 2026 phase-out schedule: 50% falling by one point for each $2,000 over $15,000 to 35%, then by one point for each $2,000 ($4,000 on a joint return) over $75,000 ($150,000) to 20%.",
    ),
    ("EXPLAIN_CARE_CREDIT_BEFORE_PRIOR", "Line 6 times the line 8 percentage."),
    ("EXPLAIN_CARE_CREDIT_TOTAL", "Lines 9a and 9b added."),
    (
        "EXPLAIN_CARE_CREDIT_LIMIT",
        "Form 1040 line 18 less the foreign tax credit: the credit cannot exceed it.",
    ),
    (
        "EXPLAIN_FORM_2441_LINE_5",
        "Your spouse’s earned income on a joint return; on any other return, your own earned income from line 4 again.",
    ),
    (
        "EXPLAIN_ADDITIONAL_MEDICARE_SELF_EMPLOYMENT",
        "Schedule 2 line 11: Additional Medicare tax on self-employment earnings, Form 8959 Part IV line 18, after the wage threshold is used first.",
    ),
    (
        "EXPLAIN_ADDITIONAL_MEDICARE_TAX",
        "Additional Medicare tax across wages and self-employment earnings.",
    ),
    (
        "EXPLAIN_ADDITIONAL_MEDICARE_WAGES",
        "Schedule 2 line 17b: Additional Medicare tax on wages and RRTA compensation, Form 8959 Part III line 12 (Part I line 7 for Medicare wages).",
    ),
    (
        "EXPLAIN_ADJUSTMENTS",
        "Half of self-employment tax, eligible owner SEP and business health insurance, educator/HSA/MSA/IRA amounts, phased student-loan interest, savings-withdrawal penalties, qualifying alimony paid, jury pay surrendered for continued salary and terminating-estate or trust section 67(e) deductions.",
    ),
    (
        "EXPLAIN_ADOPTION",
        "{0}: this employee's W-2 code T benefit for the child. The shared Form 8839 exclusion uses it once; a partly available exclusion is attributed by employee. Box 1 omits it, boxes 3 and 5 already include it, and any box 2 withholding is counted only from the W-2.",
    ),
    (
        "EXPLAIN_ADOPTION_2",
        "Actual code T excluded or taxable for this employee without changing the W-2. A special-needs deemed exclusion beyond actual code T is a separate joint Form 8839 line 31 adjustment, not employee earnings.",
    ),
    (
        "EXPLAIN_ADOPTION_BENEFITS_EXCLUDED",
        "Current exclusion, including eligible benefits taxed before a foreign adoption became final.",
    ),
    (
        "EXPLAIN_ADOPTION_CARRYFORWARD",
        "Unexpired nonrefundable balances after using the oldest credit first.",
    ),
    (
        "EXPLAIN_ADOPTION_CHILD_N_BENEFITS_N_EXCLUDED",
        "This child's own Form 8839 line 29 exclusion, before adding the other child's exclusion on line 30.",
    ),
    (
        "EXPLAIN_ADOPTION_CHILD_N_BENEFITS_N_TAXABLE",
        "This child's contribution to joint Form 8839 line 31. It is negative when a special-needs deemed exclusion exceeds actual code T.",
    ),
    (
        "EXPLAIN_ADOPTION_CHILD_N_SPECIAL_NEEDS_DEEMED_EXCESS",
        "This child's Form 8839 line 29 exclusion beyond actual W-2 code T; it reduces joint line 1f without reducing either employee's earnings.",
    ),
    (
        "EXPLAIN_ADOPTION_EXCLUSION_MAGI",
        "Form 8839 employer-benefit exclusion modified income before its own phaseout.",
    ),
    (
        "EXPLAIN_ADOPTION_JOINT_SPECIAL_NEEDS_DEEMED_EXCESS",
        "Finalized special-needs Form 8839 line 29 exclusions above actual W-2 code T. These reduce joint Form 1040 line 1f without reducing either spouse's earned income.",
    ),
    (
        "EXPLAIN_ADOPTION_NONREFUNDABLE_CREDIT",
        "Nonrefundable adoption credit after coordination with the child credit; oldest carryover is used first.",
    ),
    ("EXPLAIN_ADOPTION_N_N", "Unused 2021 nonrefundable credit expires after 2026."),
    (
        "EXPLAIN_ADOPTION_N_N_2",
        "Unused nonrefundable credit retains its original five-year carryforward period; it cannot become refundable.",
    ),
    (
        "EXPLAIN_ADOPTION_N_N_LIMIT",
        "Per-child limit assigned to this return for {0} before prior claims and income phaseout. Credit and employer-benefit limits are independent.",
    ),
    (
        "EXPLAIN_ADOPTION_PRIOR_TAXED_BENEFITS",
        "Foreign-adoption benefits received and taxed before 2026 finalization, used once in the exclusion worksheet.",
    ),
    (
        "EXPLAIN_ADOPTION_REFUNDABLE_CREDIT",
        "2026 refundable portion, up to $5,120 per child after income phaseout; prior credit carries are never refundable.",
    ),
    (
        "EXPLAIN_ADOPTION_WAGE_ADJUSTMENT",
        "Net Form 1040 line 1f wage adjustment after current benefits and the exclusion; prior-year foreign benefits or a qualified special-needs plan can make it negative. W-2 box 1 stays unchanged.",
    ),
    ("EXPLAIN_AGI", "Total income less supported adjustments."),
    (
        "EXPLAIN_MOVING_EXCESS_REIMBURSEMENT",
        "Government moving reimbursement (W-2 box 12 code P) above your Form 3903 moving expenses; taxable wages on Form 1040 line 1h.",
    ),
    (
        "EXPLAIN_MOVING_EXPENSE_DEDUCTION",
        "Form 3903 line 5: moving and travel costs of your permanent change of station or intelligence reassignment above the government reimbursement in W-2 box 12 code P. Schedule 1 line 14.",
    ),
    (
        "EXPLAIN_SCHEDULE_1_14",
        "Moving expenses for members of the Armed Forces and the intelligence community (Form 3903).",
    ),
    (
        "EXPLAIN_ALIMONY_PAID_DEDUCTION",
        "Qualifying pre-2019 alimony paid plus recipient recapture deduction. Current payments follow child-support priority; post-2018 instruments and express new-law modifications produce no current-payment deduction.",
    ),
    (
        "EXPLAIN_ALIMONY_RECAPTURE_DEDUCTION",
        "Recipient's third-year deduction for previously taxed alimony. Included in adjustments separately from current payments received.",
    ),
    (
        "EXPLAIN_ALIMONY_RECAPTURE_INCOME",
        "Payer's third-year recapture of previously deducted alimony. Included in Schedule 1 income and, for decree-related instruments, payer IRA compensation. Not EIC earnings.",
    ),
    (
        "EXPLAIN_ALTERNATIVE_MINIMUM_TAX",
        "2026 exemption, 50% phaseout, 26%/28% rates, capital-gain bands and foreign-exclusion stacking, less regular tax after the foreign credit.",
    ),
    (
        "EXPLAIN_ALTERNATIVE_MINIMUM_TAXABLE_INCOME",
        "Form 6251 line 4: regular deductions adjusted for standard deduction or SALT, senior deduction, private-activity interest, ISO exercise spread and the applicable married-separate income addition.",
    ),
    (
        "EXPLAIN_AMT_FARM_CONSERVATION_ADJUSTMENT",
        "Regular section 175 allowance minus the separately computed AMT allowance adjusts AMT business income once. Each uses its own retained carry; regular QBI and self-employment tax remain unchanged.",
    ),
    (
        "EXPLAIN_AMT_BUSINESS_DEPRECIATION_ADJUSTMENT",
        "AMT business income minus regular income from separate depreciation, before the independently recalculated excess-business-loss limit. Regular QBI and self-employment tax are unchanged.",
    ),
    (
        "EXPLAIN_AMT_CURRENT_NET_OPERATING_LOSS",
        "Current alternative-tax NOL with section 172(d) capital-loss and nonbusiness-deduction limits recomputed from AMT amounts. The standard deduction and QBI do not create an ATNOL.",
    ),
    (
        "EXPLAIN_AMT_EXCESS_BUSINESS_LOSS_CARRYFORWARD",
        "Separately calculated AMT Form 461 disallowed loss, carried to the next year in addition to the current ATNOL.",
    ),
    (
        "EXPLAIN_AMT_ISO_COMPENSATION_ADJUSTMENT",
        "Form 6251 line 2k ordinary-income adjustment for sales of stock exercised in earlier years. The separate AMT capital adjustment applies its own gain/loss limits. Same-year exercise and sale has no ISO AMT adjustment.",
    ),
    (
        "EXPLAIN_AMT_RENTAL_ADJUSTMENT",
        "AMT passive activity income minus regular income, after separate depreciation and loss allocation; do not add depreciation twice.",
    ),
    (
        "EXPLAIN_AMT_TOTAL_NOL_CARRYFORWARD",
        "Current and unexpired historical AMT NOL after current-year use and debt reduction. Retain original loss years and exception classifications separately from regular losses.",
    ),
    (
        "EXPLAIN_ANNUITY_LOSS_N",
        "Original investment less prior recovery, calculated current-year recovery and other tax-free payouts. Available only on the final annuitant's itemized final return.",
    ),
    (
        "EXPLAIN_ANNUITY_N_N_BASIS_INCREASE",
        "Taxable nonqualified-annuity loan or pledge increases investment under section 72(e)(4)(A). Its untaxed portion does not reduce investment, and repayment does not increase it again.",
    ),
    (
        "EXPLAIN_ANNUITY_N_N_BASIS_RECOVERY",
        "Final contract payment recovers remaining investment first; only cash above that investment is taxable.",
    ),
    (
        "EXPLAIN_ANNUITY_N_N_BASIS_RECOVERY_2",
        "The portion of this withdrawal exceeding available combined contract earnings is tax-free recovery of investment.",
    ),
    (
        "EXPLAIN_ANNUITY_N_N_BASIS_RECOVERY_3",
        "General Rule ratio after any refund-feature adjustment, rounded to three decimal places and applied to full and partial payments. Starts after 1986 are capped by remaining net cost before that adjustment; earlier starts continue the exclusion.",
    ),
    (
        "EXPLAIN_ANNUITY_N_N_EARLIER_RECOVERY",
        "First annuitant's earlier 2026 tax-free recovery reduces common remaining investment; it is not income or an exclusion of the survivor.",
    ),
    (
        "EXPLAIN_ANNUITY_N_N_EXPECTED_RETURN",
        "Original installment amount times the fixed installment count, or annual payments weighted by IRS Table V, VI or VIII factors. Lifetime factors include the applicable original timing adjustment; temporary-life factors do not.",
    ),
    (
        "EXPLAIN_ANNUITY_N_N_REFUND_ADJUSTMENT",
        "Single-life Table VII percentage times the lesser of original net cost or guaranteed return, rounded to whole dollars. This reduces investment for the exclusion ratio, not the net-cost recovery cap or final death deduction.",
    ),
    (
        "EXPLAIN_ANNUITY_N_N_REMAINING_BASIS",
        "Combined investment after this event; taxable loans/pledges increase it and repayments do not change it, before subsequent contributions or other basis adjustments. If related contracts are present this is a group total, not the paying contract's separate basis.",
    ),
    (
        "EXPLAIN_ANNUITY_N_N_REMAINING_BASIS_2",
        "Unrecovered contract cost after earlier first-annuitant recovery and this recipient's payments; retain the complete common record for next year's calculation.",
    ),
    (
        "EXPLAIN_ANNUITY_N_N_ROLLED_BASIS",
        "After-tax investment transferred to the receiving retirement account, if this is an eligible employer-plan rollover.",
    ),
    (
        "EXPLAIN_ANNUITY_N_N_SURRENDER_LOSS",
        "Unrecovered cost on complete cash surrender is a nondeductible miscellaneous itemized loss in 2026, not a capital loss or future contract basis.",
    ),
    (
        "EXPLAIN_ANNUITY_N_N_TAXABLE_WITHDRAWAL",
        "Actual or deemed distribution allocated first to earnings of the paying contract and required related contracts combined. Cash values ignore surrender charges.",
    ),
    (
        "EXPLAIN_ARCHER_CONTRIBUTION_N_N",
        "Form 8853 Part I: monthly 65% self-only or 75% family deductible limit, spouse allocation and linked compensation after applicable SE-tax deduction. Employer deposits are excluded, never deducted again.",
    ),
    (
        "EXPLAIN_ARCHER_CORRECTION_EARNINGS",
        "Positive earnings returned with timely Archer MSA excess corrections enter other income in the distribution year. A loss reduces the cash returned but not the principal removed.",
    ),
    (
        "EXPLAIN_ARCHER_CORRECTION_EMPLOYER_PRINCIPAL_OTHER_INCOME",
        "Timely returned employer excess principal deposited in 2026 enters 2026 other income when absent from W-2 wages; a 2027 deposit belongs to 2027 income.",
    ),
    (
        "EXPLAIN_ARCHER_MSA_ADDITIONAL_TAX",
        "20% of combined nonmedical withdrawals after the account holder age/disability exceptions. Eligible rollovers and inherited death-date values are excluded from this additional tax.",
    ),
    (
        "EXPLAIN_ARCHER_MSA_DEDUCTION",
        "Eligible personal Archer MSA contributions and retained prior excess using unused 2026 room reduce income; employer contributions do not create another deduction.",
    ),
    (
        "EXPLAIN_ARCHER_MSA_N_N",
        "Form 8853 Part II: qualified medical spending and eligible indirect rollovers are excluded. Deemed distributions cannot be excluded for medical spending or rolled over. Terminated accounts use January 1 value and date; loan pledges use collateral value on the pledge date. Retained nonmedical amounts enter income; withdrawals after age 65 or disability onset escape the 20% additional tax. Combine spouses before applying the tax rate.",
    ),
    (
        "EXPLAIN_ARCHER_MSA_TAXABLE_DISTRIBUTIONS",
        "Taxable ordinary Archer MSA withdrawals and inherited account values after applicable medical and rollover exclusions. Schedule 1 income; inherited death transfers do not incur the separate withdrawal penalty.",
    ),
    (
        "EXPLAIN_ARCHER_N_CORRECTED_CONTRIBUTIONS",
        "Timely returned 2026-designated principal is removed before Archer MSA contribution and HSA-limit calculations; attributable earnings and taxable employer principal follow their own income years.",
    ),
    (
        "EXPLAIN_ARCHER_N_EXCESS_ACCOUNT_VALUE",
        "Sum of the owners' account values used separately to cap each owner's Form 5329 tax; this sum is not a shared tax ceiling.",
    ),
    (
        "EXPLAIN_ARCHER_N_EXCESS_CONTRIBUTION_TAX",
        "This owner's Form 5329 Part VI line 41 tax, calculated separately from the spouse's excess and account value.",
    ),
    (
        "EXPLAIN_ARCHER_N_EXCESS_CONTRIBUTION_TAX_2",
        "Form 5329 Part VI line 41: 6% of the smaller of remaining prior plus current uncorrected personal/employer Archer MSA excess or owner account value, included on Schedule 2 line 18.",
    ),
    (
        "EXPLAIN_ARCHER_N_N_EXCESS_ACCOUNT_VALUE",
        "All owner Archer MSA fair market values at December 31 plus later deposits designated for 2026 cap the 6% tax.",
    ),
    (
        "EXPLAIN_ARCHER_N_N_UNCORRECTED_EMPLOYER_EXCESS",
        "Employer contributions above the smaller of the Form 8853 coverage or linked-compensation limits enter Form 5329 line 39.",
    ),
    (
        "EXPLAIN_ARCHER_N_N_UNCORRECTED_EMPLOYER_EXCESS_OTHER_INCOME",
        "Uncorrected excess from employer deposits actually made in 2026 enters Schedule 1 other income unless already included in 2026 W-2 wages. Excess from 2027 deposits belongs to 2027 income.",
    ),
    (
        "EXPLAIN_ARCHER_N_N_UNCORRECTED_PERSONAL_EXCESS",
        "Personal Archer MSA deposits not deductible under Form 8853 enter Form 5329 line 39, including deposits barred by employer contributions.",
    ),
    (
        "EXPLAIN_ARCHER_N_PRIOR_DESIGNATED_CORRECTED_CONTRIBUTIONS",
        "Timely returned 2025-designated principal belongs on the corrected 2025 Form 8853, not in 2026 contribution or HSA-offset totals. Current-year earnings and employer-principal income use their actual dates.",
    ),
    (
        "EXPLAIN_ARCHER_N_PRIOR_DESIGNATED_EMPLOYER_EXCESS",
        "Uncorrected employer excess deposited in 2026 but designated for 2025 belongs to retained Form 8853 contribution and Form 5329 excess worksheets, not 2026 contributions or HSA offsets.",
    ),
    (
        "EXPLAIN_ARCHER_N_PRIOR_EXCESS_N_DEDUCTED_N",
        "Remaining prior excess treated as a 2026 Archer MSA contribution and deducted on Form 8853 line 5 within unused contribution room.",
    ),
    (
        "EXPLAIN_ARCHER_N_PRIOR_EXCESS_N_REMAINING_N",
        "Prior excess remaining after taxable Archer MSA distributions and unused 2026 contribution room; combined with current excess for Form 5329 line 40.",
    ),
    (
        "EXPLAIN_ARCHER_N_PRIOR_EXCESS_N_USED_BY_DISTRIBUTIONS",
        "Retained 2025 Form 5329 line 40 excess absorbed by taxable 2026 Archer MSA distributions on Form 8853 line 8.",
    ),
    (
        "EXPLAIN_ARCHER_N_UNCORRECTED_EMPLOYER_EXCESS",
        "Sum of owner-specific employer Archer MSA excess on Form 5329 line 39.",
    ),
    (
        "EXPLAIN_ARCHER_N_UNCORRECTED_EMPLOYER_EXCESS_OTHER_INCOME",
        "Employer Archer MSA excess actually deposited in 2026 and omitted from W-2 wages enters 2026 other income.",
    ),
    (
        "EXPLAIN_ARCHER_N_UNCORRECTED_PERSONAL_EXCESS",
        "Sum of owner-specific personal Archer MSA excess on Form 5329 line 39.",
    ),
    (
        "EXPLAIN_ARCHER_PRIOR_DESIGNATED_EMPLOYER_EXCESS_OTHER_INCOME",
        "Uncorrected employer Archer MSA excess designated for 2025 but actually deposited in 2026 enters 2026 other income if omitted from W-2 wages; it is not a 2026 contribution.",
    ),
    (
        "EXPLAIN_ARCHER_PRIOR_EXCESS_N_DEDUCTED_N",
        "Sum of owner-specific retained Archer excess deducted using each owner's unused 2026 contribution room.",
    ),
    (
        "EXPLAIN_ARCHER_PRIOR_EXCESS_N_REMAINING_N",
        "Sum of owner-specific prior excess remaining for each owner's later Form 5329.",
    ),
    (
        "EXPLAIN_ARCHER_PRIOR_EXCESS_N_USED_BY_DISTRIBUTIONS",
        "Sum of owner-specific retained Archer excess absorbed by each owner's taxable 2026 Form 8853 line 8 distributions.",
    ),
    ("EXPLAIN_ASSET_SALE_N", "Building part of the sale price (the gross sales price)."),
    (
        "EXPLAIN_ASSET_SALE_N_2",
        "Cost of the building and improvements, without the land, plus its share of selling expenses.",
    ),
    (
        "EXPLAIN_ASSET_SALE_N_3",
        "Depreciation allowed or allowable through the month of sale, mid-month convention.",
    ),
    ("EXPLAIN_ASSET_SALE_N_4", "Line 23 less line 24."),
    ("EXPLAIN_ASSET_SALE_N_5", "Line 22 less line 25."),
    ("EXPLAIN_ASSET_SALE_N_6", "Business-use share of the sale price (the gross sales price)."),
    (
        "EXPLAIN_ASSET_SALE_N_7",
        "Business-use share of the cost, plus its selling expenses. A clean-vehicle or other credit that lowered the basis is not subtracted here; it is part of line 24.",
    ),
    (
        "EXPLAIN_ASSET_SALE_N_8",
        "Depreciation, section 179, special allowances and the basis reduction of any credit (a clean-vehicle or investment credit), allowed or allowable, including 2026 by the convention for the year of disposition.",
    ),
    (
        "EXPLAIN_ATNOL_N_N_CARRYFORWARD",
        "Unused ATNOL continuing into 2027 after debt reduction; keep its original loss year and statutory exception classification.",
    ),
    (
        "EXPLAIN_ATNOL_N_N_DEBT_REDUCTION",
        "AMT Form 982 reduction after current tax, current-year loss first and then original-year order.",
    ),
    (
        "EXPLAIN_ATNOL_N_N_EXPIRED",
        "Unused pre-2018 loss beyond its twenty-year carryforward period, including a 2006 balance expiring after 2026.",
    ),
    (
        "EXPLAIN_ATNOL_N_N_OPENING",
        "Retained ATNOL balance entering 2026, separate from regular NOL.",
    ),
    (
        "EXPLAIN_ATNOL_N_N_USED",
        "Absorbed by modified 2026 AMTI, oldest loss year first subject to the ordinary 90% limit and documented exceptions. This can exceed the deduction claimed.",
    ),
    (
        "EXPLAIN_BAD_DEBT_N_SHORT_TERM_LOSS",
        "The unrecovered adjusted basis of a wholly worthless nonbusiness debt is a short-term capital loss regardless of holding period. A purchased debt uses its acquisition cost, not its face amount. An insolvent personal contractor's unreturned deposit or a former-home mortgage shortfall can also qualify. Capital netting and annual loss limits apply. Retain the debt and collection statement required with Form 8949.",
    ),
    (
        "EXPLAIN_BIRTH_ADOPTION_USED_N_N",
        "Qualified birth/adoption amount used for this parent and child through 2026, including prior-year usage.",
    ),
    (
        "EXPLAIN_BOND_CREDIT_DEEMED_INTEREST",
        "Current Form 1097-BTC box 1 credit is deemed taxable interest on its 2026 credit allowance or payment dates, including the part Form 8912 cannot use. Prior-year credit carry is excluded. The amount joins Form 1040 line 2b and applicable investment-income calculations once.",
    ),
    (
        "EXPLAIN_BUSINESS_CREDIT_N_CARRYBACK_N",
        "Unused 2026 credit applied on a carryback return (the year is in the row).",
    ),
    (
        "EXPLAIN_BUSINESS_CREDIT_N_EXPIRED",
        "Unused credit at the end of its 20-year carryforward period; a qualified business credit becomes a section 196 deduction in 2027.",
    ),
    (
        "EXPLAIN_BUSINESS_CREDIT_N_TO_N",
        "Form 8582-CR: the part of this passive credit not allowed for 2026 (Worksheet 8); it carries to 2027 as a prior-year unallowed credit.",
    ),
    (
        "EXPLAIN_BUSINESS_CREDIT_N_ALLOWED",
        "Form 8582-CR line 37 share (Worksheet 9): the part of this passive credit allowed for 2026, which Form 3800 uses like a nonpassive credit.",
    ),
    (
        "EXPLAIN_BUSINESS_CREDIT_N_TO_N_2",
        "Credit carried forward to 2027 (Part IV column (i) for a carryforward).",
    ),
    (
        "EXPLAIN_BUSINESS_CREDIT_N_USED",
        "Credit used against 2026 tax, earliest year first (Part IV column (g) for a carryforward).",
    ),
    (
        "EXPLAIN_BUSINESS_NONQUALIFIED_EXCESS_LOSS",
        "Foreign service-business excess loss included once in the NOL carryforward. None is qualified business income loss, either currently or when this suspended loss is later deducted. This character detail is not an additional carryforward amount.",
    ),
    (
        "EXPLAIN_BUSINESS_N_ASSET_N_LISTED_N_BUSINESS_USE",
        "Business-use percentage: business miles divided by all miles.",
    ),
    ("EXPLAIN_BUSINESS_N_ASSET_N_LISTED_N_COST", "Cost or other basis of the whole vehicle."),
    (
        "EXPLAIN_BUSINESS_N_ASSET_N_LISTED_N_DEPRECIATION",
        "Depreciation deduction after the section 280F passenger-automobile limit on the business-use share, which also counts section 179 and the special allowance. Use 50% or less is ADS straight line over 5 years.",
    ),
    (
        "EXPLAIN_BUSINESS_N_ASSET_N_LISTED_N_DEPRECIATION_BASIS",
        "Basis for depreciation: business-use share of cost less elected section 179 and the special allowance.",
    ),
    (
        "EXPLAIN_BUSINESS_N_ASSET_N_LISTED_RECAPTURE",
        "Form 4797 Part IV line 37, column (b): the excess depreciation, which section 280F(b)(2) includes in income now that business use is 50% or less. It is on Schedule C line 6, and the vehicle's basis is increased by it.",
    ),
    (
        "EXPLAIN_BUSINESS_N_ASSET_N_LISTED_RECOMPUTED",
        "Form 4797 Part IV line 36, column (b): the straight-line (ADS) depreciation the earlier years would have allowed at the earlier business use, within the section 280F limits.",
    ),
    (
        "EXPLAIN_BUSINESS_N_ASSET_N_LISTED_PRIOR_DEPRECIATION",
        "Form 4797 Part IV line 35, column (b): the depreciation, with any section 179 and bonus, allowed for the earlier years while the vehicle was used more than 50% for business.",
    ),
    (
        "EXPLAIN_BUSINESS_N_ASSET_N_LISTED_N_SECTION179",
        "Elected section 179 cost; zero unless business use is more than 50%.",
    ),
    (
        "EXPLAIN_BUSINESS_N_ASSET_N_LISTED_N_TYPE",
        "Type of property: {1}; vehicle {0}, listed first.",
    ),
    (
        "EXPLAIN_BUSINESS_N_ASSET_N_LISTED_SPECIAL_ALLOWANCE",
        "Special depreciation allowance for this vehicle used more than 50% for business, after the section 280F limit.",
    ),
    (
        "EXPLAIN_BUSINESS_N_ASSET_N_N_AMT_DEPRECIATION",
        "Current AMT deduction from the retained asset schedule, using the same recovery period and convention with 150% declining balance when required. This does not change regular self-employment tax or QBI.",
    ),
    (
        "EXPLAIN_BUSINESS_N_ASSET_N_N_AMT_REMAINING_BASIS",
        "Separately retained AMT basis after current depreciation; do not replace regular asset basis.",
    ),
    (
        "EXPLAIN_BUSINESS_N_ASSET_N_N_BONUS",
        "Bonus allowance on business-use basis after elected section 179 cost: 100% for qualifying acquisition after January 19, 2025; 20% for ordinary eligible acquisition after September 27, 2017 and before January 20, 2025; zero if elected out or already placed in service before 2026. This is not the total deduction when basis remains for MACRS.",
    ),
    (
        "EXPLAIN_BUSINESS_N_ASSET_N_N_MACRS",
        "Current GDS deduction after elected section 179 and bonus. New assets use the class-wide method and return-wide convention; continuing assets preserve their original convention and table/manual approach, including the required straight-line switch. Add to bonus once. Bonus-eligible property placed after 2015 has the same regular/AMT depreciation even when bonus is elected out; post-1998 150%/straight-line GDS also agrees when basis history is identical. Retain basis and method records.",
    ),
    (
        "EXPLAIN_BUSINESS_N_ASSET_N_N_REMAINING_BASIS",
        "Business depreciable basis after prior allowed-or-allowable depreciation, elected section 179 (including separately reported unused carryforward), current bonus and current MACRS, before future events. This does not reduce original acquisition basis used for QBI property limits.",
    ),
    (
        "EXPLAIN_BUSINESS_N_ASSET_N_N_SECTION179",
        "Current section 179 deduction after shared dollar and business-income limits: current elections first, then oldest carryforward year; proportionate asset allocation within each year. Deducted once in this business.",
    ),
    (
        "EXPLAIN_BUSINESS_N_ASSET_N_N_SECTION179_CARRYFORWARD",
        "Unused asset-specific section 179 amount. Retain with the original election and asset records. It is excluded from bonus/MACRS basis; do not deduct it again as depreciation or an ordinary expense. Disposal or changed use requires separate treatment.",
    ),
    (
        "EXPLAIN_BUSINESS_N_CARD_ISSUER_RETAINED_SCHEDULE_C_N",
        "Schedule C line 6 other income: retained registered-card-issuer fuel-tax benefit accrued in 2026 after the buyer funded the claimable tax cost and the issuer obtained an unconditional right to the separate claim. Inclusion is an inference from section 61 business gain and the complete card ledger; it does not rely on the conditional deducted-tax recovery rule. Buyer card payments and vendor settlements are balance-sheet flows, not additional business receipts or expenses. The same Form 4136 amount separately reaches Schedule 3 line 12 as a refundable payment; neither amount is entered again elsewhere.",
    ),
    (
        "EXPLAIN_BUSINESS_N_CAR_AND_TRUCK",
        "Car and truck expenses: standard-mileage vehicles plus the business-mile share of actual-expense vehicles' operating costs. Vehicle-loan interest is on its own line (Schedule C 16b, Schedule F 21b) and actual-expense depreciation is on the depreciation line, not here.",
    ),
    (
        "EXPLAIN_BUSINESS_N_CASH_FUEL_TAX_RECOVERY",
        "Cash-method 2026 Schedule F line 8 other income from a prior-year Form 4136 credit or fuel-tax refund after any documented section 111 exclusion. For full prior tax benefit, this equals the gross amount. The same taxable amount must not also be in recorded receipts.",
    ),
    (
        "EXPLAIN_BUSINESS_N_CASH_FUEL_TAX_RECOVERY_2",
        "Cash-method 2026 Schedule C line 6 other income from a prior-year Form 4136 credit or fuel-tax refund after any documented section 111 exclusion. For full prior tax benefit, this equals the gross amount. The same taxable amount must not also be in recorded receipts.",
    ),
    (
        "EXPLAIN_BUSINESS_N_CASH_FUEL_TAX_RECOVERY_EXCLUDED",
        "Section 111 amount excluded from 2026 Schedule C income: the recovered tax less the increase in 2025 taxable income when that one deduction is removed. The retained comparison includes related half-SE and QBI changes; original taxable income was zero. No unused credits or carryovers change the result.",
    ),
    (
        "EXPLAIN_BUSINESS_N_CASH_FUEL_TAX_RECOVERY_GROSS",
        "Gross 2026 cash-method fuel-tax credit or refund before the section 111 tax-benefit exclusion. It was deducted on the original 2025 Schedule C, not claimed as a second 2026 Form 4136 payment.",
    ),
    (
        "EXPLAIN_BUSINESS_N_COST_OF_GOODS_SOLD",
        "Opening inventory plus purchases and production costs, less personal withdrawals and closing inventory; deducted once from business receipts.",
    ),
    (
        "EXPLAIN_BUSINESS_N_EXCLUDED_CHILD_PREMIUMS",
        "Unused incremental premiums for a child under 27 who is not eligible for medical itemization. No Schedule A deduction is allowed for this amount.",
    ),
    (
        "EXPLAIN_BUSINESS_N_EXPENSE_LINE10_COMMISSIONS_AND_FEES",
        "Schedule C line 10: commissions and fees.",
    ),
    ("EXPLAIN_BUSINESS_N_EXPENSE_LINE11_CONTRACT_LABOR", "Schedule C line 11: contract labor."),
    (
        "EXPLAIN_BUSINESS_N_EXPENSE_LINE14_EMPLOYEE_BENEFIT_PROGRAMS",
        "Schedule C line 14: employee benefit programs, other than retirement plans reported on line 19.",
    ),
    (
        "EXPLAIN_BUSINESS_N_EXPENSE_LINE15_INSURANCE",
        "Schedule C line 15: insurance, other than health.",
    ),
    (
        "EXPLAIN_BUSINESS_N_EXPENSE_LINE16A_MORTGAGE_INTEREST",
        "Schedule C line 16a: mortgage interest paid to banks or other financial institutions.",
    ),
    (
        "EXPLAIN_BUSINESS_N_EXPENSE_LINE16C_OTHER_INTEREST",
        "Schedule C line 16c: other business interest; interest on a vehicle loan is on line 16b.",
    ),
    (
        "EXPLAIN_BUSINESS_N_EXPENSE_LINE17_LEGAL_AND_PROFESSIONAL",
        "Schedule C line 17: legal and professional services.",
    ),
    ("EXPLAIN_BUSINESS_N_EXPENSE_LINE18_OFFICE", "Schedule C line 18: office expense."),
    (
        "EXPLAIN_BUSINESS_N_EXPENSE_LINE20A_RENT_VEHICLES_EQUIPMENT",
        "Schedule C line 20a: rent or lease of vehicles, machinery and equipment.",
    ),
    (
        "EXPLAIN_BUSINESS_N_EXPENSE_LINE20B_RENT_OTHER",
        "Schedule C line 20b: rent or lease of other business property.",
    ),
    ("EXPLAIN_BUSINESS_N_EXPENSE_LINE21_REPAIRS", "Schedule C line 21: repairs and maintenance."),
    (
        "EXPLAIN_BUSINESS_N_EXPENSE_LINE22_SUPPLIES",
        "Schedule C line 22: supplies not included in cost of goods sold.",
    ),
    (
        "EXPLAIN_BUSINESS_N_EXPENSE_LINE23_TAXES_AND_LICENSES",
        "Schedule C line 23: taxes and licenses.",
    ),
    ("EXPLAIN_BUSINESS_N_EXPENSE_LINE24A_TRAVEL", "Schedule C line 24a: travel."),
    (
        "EXPLAIN_BUSINESS_N_EXPENSE_LINE24B_MEALS",
        "Schedule C line 24b: deductible meals, after the 50% ordinary or 80% DOT hours-of-service limit.",
    ),
    ("EXPLAIN_BUSINESS_N_EXPENSE_LINE25_UTILITIES", "Schedule C line 25: utilities."),
    (
        "EXPLAIN_BUSINESS_N_EXPENSE_LINE26_WAGES",
        "Schedule C line 26: wages, less any employment credits.",
    ),
    (
        "EXPLAIN_BUSINESS_N_EXPENSE_LINE27B_OTHER",
        "Schedule C line 27b: other expenses from Part V, including a section 481(a) decrease.",
    ),
    ("EXPLAIN_BUSINESS_N_EXPENSE_LINE8_ADVERTISING", "Schedule C line 8: advertising."),
    (
        "EXPLAIN_BUSINESS_N_FOREIGN_NET_INCOME",
        "Wholly foreign service-business profit or loss before any separate earned-income exclusion. SE tax uses full profit; this income and its attributable deductions do not enter U.S. qualified business income.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM5884_LINE0_1A",
        "Form 5884 line 1a: 25% of qualified first-year wages of employees who worked 120 to 399 hours, each within its targeted-group limit less wages counted on the 2025 form.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM5884_LINE0_1B",
        "Form 5884 line 1b: 40% of qualified first-year wages of employees who worked at least 400 hours.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM5884_LINE0_1C",
        "Form 5884 line 1c: 50% of qualified second-year wages of long-term family assistance recipients, up to $10,000 each.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM5884_LINE0_N",
        "Form 5884 lines 2 and 4 and Form 3800 Part III line 4b; the business's wages deduction is reduced by this amount.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM8846_N_ACTUAL_EMPLOYER_TIP_FICA_PAID",
        "Employer Social Security and Medicare tax actually paid or incurred on reported customer tips, reconciled from payroll; payroll-period rounding may differ from 7.65% of annual rounded tips.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM8846_N_LINE1",
        "2026 Form 8846 line 1: customer tips for eligible services reported to the employer and subject to employer Social Security and Medicare tax.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM8846_N_LINE2",
        "2026 Form 8846 line 2: monthly employee tips used to bring wages excluding tips to the selected service's statutory hourly basis.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM8846_N_LINE3",
        "2026 Form 8846 line 3: taxed customer tips after the monthly wage shortfall for the selected service.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM8846_N_LINE4",
        "2026 Form 8846 line 4: 7.65% of creditable tips; no employee reaches the 2026 Social Security wage base.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM8846_N_REMAINING_EMPLOYER_FICA_DEDUCTION",
        "Schedule C deduction equals actual employer FICA on reported tips less the determined Form 8846 line 4 credit, even if Form 3800 limits this year's credit use. Other payroll costs remain in Business expenses.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM8881_LINE_N",
        "Form 8881 line 1: qualified start-up costs paid in 2026.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM8881_LINE_N_2",
        "Form 8881 line 2: 100% of line 1 with 50 or fewer employees on line A, 50% with 51 to 100.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM8881_LINE_N_3",
        "Form 8881 line 3: $250 for each eligible non-highly compensated employee.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM8881_LINE_N_4",
        "Form 8881 line 4: the greater of $500 or line 3, not more than $5,000.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM8881_LINE_N_5",
        "Form 8881 lines 5 and 8 and Form 3800 Part III line 1j; the start-up costs deduction is reduced by this amount.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM8881_LINE_N_6",
        "Form 8881 lines 9 and 11 and Form 3800 Part III line 1dd: $500 in the first auto-enrollment year and the next two.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM8941_LINE_1",
        "Form 8941 line 1: the individuals you considered employees in 2026, seasonal workers included; excluded owners and relatives are not listed.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM8941_LINE_2",
        "Form 8941 line 2: hours of service of the listed employees divided by 2,080, rounded down (at least 1); seasonal workers' hours are left out. At 25 or more, the credit is zero.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM8941_LINE_3",
        "Form 8941 line 3: W-2 box 5 wages of the listed employees divided by line 2, rounded down to a multiple of $1,000; seasonal workers' wages are left out. At $69,000 or more, the credit is zero.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM8941_LINE_4",
        "Form 8941 line 4: premiums you paid for the employees' coverage (your share only; Worksheet 4 column (b)).",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM8941_LINE_5",
        "Form 8941 line 5: what you would have paid at your share for the months enrolled if each plan cost the county's 2026 average premium (Worksheet 4 column (c)).",
    ),
    ("EXPLAIN_BUSINESS_N_FORM8941_LINE_6", "Form 8941 line 6: the smaller of line 4 and line 5."),
    ("EXPLAIN_BUSINESS_N_FORM8941_LINE_7", "Form 8941 line 7: 50% of line 6."),
    (
        "EXPLAIN_BUSINESS_N_FORM8941_LINE_8",
        "Form 8941 line 8: line 7 reduced by (line 2 − 10) / 15 of line 7 above 10 full-time equivalent employees (Worksheet 5).",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM8941_LINE_9",
        "Form 8941 line 9: line 8 reduced by (line 3 − $34,100) / $34,100 of line 7 above $34,000 average wages, Rev. Proc. 2025-32 (Worksheet 6).",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM8941_LINE_10",
        "Form 8941 line 10: state premium subsidies and state tax credits for the line 4 premiums.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM8941_LINE_11",
        "Form 8941 line 11: line 4 less line 10, not below zero.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM8941_LINE_12",
        "Form 8941 line 12: the smaller of line 9 and line 11, the credit; Schedule C line 14 is reduced by this amount.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM8941_LINE_13",
        "Form 8941 line 13: employees enrolled in the coverage (shown when line 12 is more than zero).",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM8941_LINE_14",
        "Form 8941 line 14: full-time equivalent employees of the enrolled employees, rounded down (shown when line 12 is more than zero).",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM8941_LINE_15",
        "Form 8941 line 15: the credit from your partnerships and S corporations (Schedule K-1 code BA); it also reaches Form 3800 Part III line 4h on its own.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM8941_LINE_16",
        "Form 8941 line 16: lines 12 and 15 to Form 3800 Part III line 4h.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM8994_LINE_1A",
        "Form 8994 line 1a: for each employee, the leave wages (at most the normal hourly rate times the leave hours, and at most 12 weeks of leave) times 12.5% plus 0.25 point for each point the policy's rate of payment exceeds 50% (at most 25%); Schedule C line 26 is reduced by this amount.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM8994_LINE_1B",
        "Form 8994 line 1b: the creditable paid leave insurance premiums times the same percentage for the policy's rate of payment; the Schedule C line 14 employee benefit programs deduction is reduced by this amount.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM8994_LINE_2",
        "Form 8994 line 2: the credit from your partnerships and S corporations (Schedule K-1 code BB); it also reaches Form 3800 Part III line 4j on its own.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM8994_LINE_3",
        "Form 8994 line 3: lines 1a, 1b and 2 to Form 3800 Part III line 4j.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM_8829_ALLOWED",
        "Allowable expenses for business use of the home: lines 14, 27 and 33 (line 35 is zero). Schedule C line 30.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM_8829_BASIS",
        "The smaller of the home's cost or other basis and its value when first used for business, with the land.",
    ),
    ("EXPLAIN_BUSINESS_N_FORM_8829_BUILDING", "Basis of the building: line 37 less line 38."),
    ("EXPLAIN_BUSINESS_N_FORM_8829_DEPRECIATION", "Depreciation of the home from line 42."),
    (
        "EXPLAIN_BUSINESS_N_FORM_8829_DEPRECIATION_ALLOWED",
        "Allowable excess casualty losses and depreciation: the smaller of line 28 or line 32.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM_8829_DEPRECIATION_CARRYOVER",
        "Excess casualty losses and depreciation carried over to 2027: line 32 less line 33.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM_8829_DEPRECIATION_LIMIT",
        "Limit on excess casualty losses and depreciation: line 15 less line 27.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM_8829_DEPRECIATION_TOTAL",
        "Lines 29 through 31; casualty losses are not calculated.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM_8829_DIRECT_REPAIRS",
        "Repairs and painting of the office space only, column (a).",
    ),
    ("EXPLAIN_BUSINESS_N_FORM_8829_DIRECT_TOTAL", "Lines 16 through 22, column (a)."),
    (
        "EXPLAIN_BUSINESS_N_FORM_8829_DEDUCTIBLE_INDIRECT",
        "Lines 9 through 11, column (b); the business share of it is line 13.",
    ),
    ("EXPLAIN_BUSINESS_N_FORM_8829_DEDUCTIBLE_DIRECT", "Lines 9 through 11, column (a)."),
    (
        "EXPLAIN_BUSINESS_N_FORM_8829_EXCESS_INTEREST",
        "Standard deduction: all mortgage interest and mortgage insurance premiums on loans to buy, build or improve the home, column (b).",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM_8829_EXCESS_TAXES",
        "Itemizing: the business share of real estate taxes above the state and local tax limit room, column (a); standard deduction: all the home's real estate taxes, column (b).",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM_8829_INCOME",
        "Schedule C line 29 less this business's Form 4797 losses on its assets, which do not come from the use of the home.",
    ),
    ("EXPLAIN_BUSINESS_N_FORM_8829_INDIRECT_TOTAL", "Lines 16 through 22, column (b)."),
    ("EXPLAIN_BUSINESS_N_FORM_8829_INSURANCE", "Insurance, column (b)."),
    ("EXPLAIN_BUSINESS_N_FORM_8829_LAND", "The land included on line 37."),
    ("EXPLAIN_BUSINESS_N_FORM_8829_LIMIT", "Line 8 less line 14, not less than zero."),
    (
        "EXPLAIN_BUSINESS_N_FORM_8829_MORTGAGE_INTEREST",
        "Itemizing: the home's deductible mortgage interest and mortgage insurance premiums, column (b). The personal part stays on Schedule A lines 8a and 8d.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM_8829_OPERATING_ALLOWED",
        "Allowable operating expenses: the smaller of line 15 or line 26.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM_8829_OPERATING_CARRYOVER",
        "Operating expenses carried over to 2027: line 26 less line 27.",
    ),
    ("EXPLAIN_BUSINESS_N_FORM_8829_OPERATING_TOTAL", "Line 23 column (a) plus lines 24 and 25."),
    ("EXPLAIN_BUSINESS_N_FORM_8829_OTHER", "Other expenses of the whole home, column (b)."),
    (
        "EXPLAIN_BUSINESS_N_FORM_8829_PRIOR_DEPRECIATION",
        "Excess casualty losses and depreciation carried over from the last Form 8829 (its line 44).",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM_8829_PRIOR_OPERATING",
        "Operating expenses carried over from the last Form 8829 (its line 43).",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM_8829_REAL_ESTATE_TAXES",
        "Itemizing: when your state and local taxes together are within $10,000 ($5,000 if married filing separately), all the home's real estate taxes, column (b) (Step 1); otherwise Line 11 Worksheet line 10, the business share of those taxes up to the room left under the state and local tax limit, column (a) (Step 2).",
    ),
    ("EXPLAIN_BUSINESS_N_FORM_8829_RENT", "Rent, column (b)."),
    (
        "EXPLAIN_BUSINESS_N_FORM_8829_REPAIRS",
        "Repairs and maintenance of the whole home, column (b).",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM_8829_UNLIMITED",
        "Line 12 column (a) plus line 13: deductible without the income limit.",
    ),
    ("EXPLAIN_BUSINESS_N_FORM_8829_UTILITIES", "Utilities, column (b)."),
    (
        "EXPLAIN_BUSINESS_N_FORM_8829_WORKSHEET_BUSINESS_TAXES",
        "Itemizing: Line 11 Worksheet line 6, the business share of the home's real estate taxes, left out of Schedule A line 5b.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FUEL_N",
        "2026 Form 4136 qualified fuel quantities at the printed rate or the authorized line 13c 24.4-cent-tax write-in rate, using GGE or DGE on applicable alternative-fuel rows and rounding to cents on this business's form line. If full fuel cost was deducted, the credit enters gross income in the fuel-use year for accrual accounting or the filing year for cash accounting.",
    ),
    (
        "EXPLAIN_BUSINESS_N_HEALTH_INSURANCE",
        "Eligible coverage months, limited to this business's profit or attributable optional SE earnings less its share of deductible self-employment tax and its owner SEP deduction. This is an income adjustment, not a Schedule C expense.",
    ),
    (
        "WARN_INTANGIBLE_BENEFIT_LEFT_OUT",
        "{0}: This estimate leaves out the uncomputed intangible deduction and keeps the activity's known income. {1} Accepted records are retained; correct the actual source through normal editing. Any section 179 income limit is estimated without this deduction and may change when these records are resolved.",
    ),
    (
        "EXPLAIN_BUSINESS_N_INTANGIBLE_N_N_AMORTIZATION",
        "Section 197: original business cost amortized over 180 months from the later of acquisition and business commencement. Later acquisition costs use the remaining original months, or are deducted immediately after that period expires. This line deducts only 2026 amounts; no initial $5,000 deduction or bonus depreciation applies.",
    ),
    (
        "EXPLAIN_BUSINESS_N_INTANGIBLE_N_N_PERIOD",
        "Form 4562-B line 1 column (e): the 180-month (15-year) amortization period of section 197(a).",
    ),
    (
        "EXPLAIN_BUSINESS_N_INTANGIBLE_N_N_AMORTIZABLE",
        "Form 4562-B line 1: the original and later capitalized cost of {0}, amortized under section 197 beginning with the month of {1}.",
    ),
    (
        "EXPLAIN_BUSINESS_N_INTANGIBLE_N_N_REMAINING_BASIS",
        "Unrecovered business intangible basis after 2026 amortization. Retain original allocation and dates for future deductions and disposition. This intangible is not tangible qualified property for QBI UBIA.",
    ),
    (
        "EXPLAIN_BUSINESS_N_KIDDIE_EARNED_INCOME",
        "Personal-service business earnings for the young-filer worksheets. Mixed-source profit uses the same attributable half-SE deduction as QBI before its 30% ceiling; a loss has no 30% ceiling.",
    ),
    (
        "EXPLAIN_BUSINESS_N_LONG_TERM_CARE_DEDUCTION",
        "Qualified long-term-care part of this business's health deduction, within remaining shared per-person age limits. Included in the total health adjustment, not an additional deduction.",
    ),
    (
        "EXPLAIN_BUSINESS_N_MACRS_MID_QUARTER",
        "Convention indicator: 1 = mid-quarter, 0 = half-year. The >40% test combines all entered 2026 equipment across the return before bonus depreciation, after personal-use allocation and elected section 179 cost. MACRS preserves each asset’s table/manual computation choice; retain that approach. File class-wide bonus election-out and any optional method elections with the timely return.",
    ),
    (
        "EXPLAIN_BUSINESS_N_MARKETPLACE_HEALTH_INSURANCE",
        "Canonical Marketplace premiums established under this business. Nonspecified premiums use its existing earnings limit first; specified premiums use the remaining limit and selected shared premium-credit method. Each source and the same allocable credit are counted once in this business's health deduction.",
    ),
    (
        "EXPLAIN_BUSINESS_N_MEDICAL_PREMIUMS",
        "Remaining medical-eligible premiums enter itemization once, subject to the 7.5% AGI floor. The self-employed deduction uses eligible incremental nondependent-child premiums first; unused child-only amounts do not enter Schedule A.",
    ),
    (
        "EXPLAIN_BUSINESS_N_NET_PROFIT",
        "Net profit or loss: tentative profit less the business use of your home. It enters Schedule 1 line 3 and Schedule SE line 2.",
    ),
    (
        "EXPLAIN_BUSINESS_N_N_HOME_OFFICE",
        "Business-use-of-home deduction allocated to this activity from its linked home claims. The Home offices page shows each claim's method, income limits, and carryforwards.",
    ),
    (
        "EXPLAIN_BUSINESS_N_N_HOME_OFFICE_2",
        "Simplified home-office deduction: monthly qualifying area at $5 per square foot annually, limited to positive business income; no current-year unused amount carries forward.",
    ),
    ("EXPLAIN_BUSINESS_N_N_N", "Gross receipts or sales."),
    (
        "EXPLAIN_BUSINESS_N_N_N_2",
        "Other business income: a prior fuel-tax credit or refund recovered, fuel-credit income under the accrual method, and the 2026 part of a section 481(a) increase.",
    ),
    (
        "EXPLAIN_BUSINESS_N_N_N_3",
        "Gross income: gross receipts less cost of goods sold, plus other income.",
    ),
    (
        "EXPLAIN_BUSINESS_N_N_N_4",
        "Depreciation and section 179 expense from Form 4562: bonus, MACRS and the allowed section 179 deduction.",
    ),
    ("EXPLAIN_BUSINESS_N_N_N_5", "Total expenses before the business use of your home."),
    ("EXPLAIN_BUSINESS_N_N_N_6", "Tentative profit or loss: gross income less total expenses."),
    (
        "EXPLAIN_BUSINESS_N_N_N_7",
        "Section 179 expense deduction allowed to this business after the dollar and business-income limits.",
    ),
    (
        "EXPLAIN_BUSINESS_N_N_N_8",
        "Total depreciation and section 179: bonus, MACRS for 2026 and earlier assets, listed property and the allowed section 179 deduction.",
    ),
    (
        "EXPLAIN_BUSINESS_N_N_OFFICE_DEPRECIATION_CARRY",
        "Prior actual depreciation/casualty carryforward is preserved; simplified use creates no current home depreciation deduction.",
    ),
    (
        "EXPLAIN_BUSINESS_N_N_OFFICE_OPERATING_CARRY",
        "Prior actual operating expenses remain available for a future actual-expense year; not deducted this year.",
    ),
    (
        "EXPLAIN_BUSINESS_N_QUALIFIED_OVERTIME",
        "FLSA overtime premium separately reported on this business's 2026 Form 1099 and already included in ordinary receipts. It shares the Schedule 1-A annual cap and income phaseout with W-2 overtime, without reducing Schedule C profit or self-employment tax.",
    ),
    (
        "EXPLAIN_BUSINESS_N_QUALIFIED_TIPS",
        "Schedule 1-A business tip amount after this business's separate net-income limit, including allocable half-SE, owner retirement and health deductions. The shared $25,000 return cap and income phaseout apply to employee and business tips together; this line does not reduce Schedule C profit or self-employment tax.",
    ),
    (
        "EXPLAIN_BUSINESS_N_SEP_DEDUCTION",
        "Timely employer contribution for this owner's traditional SEP. Reduces AGI, QBI, IRA compensation and this business's health-insurance income limit, but not SE tax.",
    ),
    (
        "EXPLAIN_BUSINESS_N_SEP_LIMIT",
        "2026 traditional SEP ceiling: net business earnings after deductible SE tax times the reduced plan rate, limited to the original plan rate times $360,000 compensation and $72,000. Deposit eligibility is checked in cents.",
    ),
    (
        "EXPLAIN_BUSINESS_N_STARTUP_N_AMORTIZATION",
        "Remaining qualifying costs are amortized over 180 months beginning with the opening month. Only 2026 months are deducted; cumulative cent rounding preserves total basis. Prior-year initial deductions are not repeated.",
    ),
    (
        "EXPLAIN_BUSINESS_N_STARTUP_PERIOD",
        "Form 4562-B line 1 column (e): the 180-month amortization period of section 195(b)(1)(B).",
    ),
    (
        "EXPLAIN_BUSINESS_N_STARTUP_AMORTIZABLE",
        "Form 4562-B line 1: the qualifying start-up costs left after the initial deduction, amortized under section 195 over 180 months beginning with the month the business began ({0}).",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM_4562B_TOTAL",
        "Form 4562-B line 3: line 1 column (f) plus line 2. It is carried to Schedule C Part V (lines 48 and 27b) with the start-up costs' initial deduction, because the 2026 Form 4562 has no amortization part.",
    ),
    (
        "EXPLAIN_BUSINESS_N_FORM_4562B_EARLIER",
        "Form 4562-B line 2: this year's amortization of start-up costs and section 197 intangibles whose amortization began before 2026. Only amortization that begins in 2026 is listed on line 1.",
    ),
    (
        "EXPLAIN_BUSINESS_N_STARTUP_N_INITIAL",
        "Initial start-up deduction in the opening year: at most $5,000, reduced dollar-for-dollar by total costs over $50,000. Capitalization election means no deduction.",
    ),
    (
        "EXPLAIN_BUSINESS_N_STARTUP_N_REMAINING_BASIS",
        "Unrecovered start-up basis after 2026. An affirmative capitalization election retains the entire cost with no amortization; this irrevocable original election applies to all start-up costs of this business.",
    ),
    (
        "EXPLAIN_BUSINESS_N_VEHICLE_N_BASIS_REDUCTION",
        "Owned vehicle: 35 cents per business mile is the depreciation component for basis records, not an additional deduction. Leased vehicles have no owner basis reduction.",
    ),
    (
        "EXPLAIN_BUSINESS_N_VEHICLE_N_EXPENSE",
        "2026 business miles use 72.5 cents through June and 76 cents from July; add business parking/tolls and allocated interest/value-based tax once.",
    ),
    (
        "EXPLAIN_BUSINESS_N_VEHICLE_LOAN_INTEREST",
        "Business-use share of vehicle-loan interest (annual interest times business miles over total miles), shown on its own line in the 2026 draft Schedule C (line 16b) and Schedule F (line 21b) whether the vehicle uses the standard mileage rate or actual costs. It is not also in car and truck expenses; the Schedule 1-A vehicle-loan-interest deduction cannot be claimed again for it.",
    ),
    (
        "EXPLAIN_BUSINESS_N_WAGERING_DEDUCTION",
        "2026 section 165(d): 90% of wagering losses plus otherwise deductible wagering expenses, capped at gross winnings. Applied before business profit, SE tax and QBI.",
    ),
    (
        "EXPLAIN_BUSINESS_QUALIFIED_EXCESS_LOSS",
        "Domestic qualified excess loss included once in the NOL carryforward, after offsetting foreign business profit. Excluded from current QBI losses and retained as qualified suspended loss for the year it is deducted; not an additional carryforward amount.",
    ),
    (
        "EXPLAIN_CAPITAL_GAIN",
        "Net short- and long-term gains; apply the current-year ordinary loss limit. Carryforward output is not a filed schedule.",
    ),
    (
        "EXPLAIN_CAPITAL_GAIN_11",
        "Long-term gain from Form 6781 line 9 (60% of section 1256 contracts) plus long-term gain recognized this year from earlier-year installment sales of investment property (Form 6252 line 26) and from earlier related-party exchanges disposed of within two years (Form 8824 line 24), plus long-term personal casualty or theft gain (Form 4684 line 15), Form 2439 box 1a undistributed gain and the Form 4797 Part I gain, as Schedule D line 11.",
    ),
    (
        "EXPLAIN_CAPITAL_GAIN_1A",
        "Totals for short-term transactions reported on a Form 1099-B or 1099-DA showing basis reported to the IRS, with no Form 8949 adjustment code and no wash-sale disallowance (Form 8949 box A or G, no adjustment).",
    ),
    (
        "EXPLAIN_CAPITAL_GAIN_1B",
        "Totals for short-term transactions reported on Form 8949 with box A or G checked: basis reported to the IRS, but with an adjustment code or a wash-sale disallowance.",
    ),
    (
        "EXPLAIN_CAPITAL_GAIN_4",
        "Short-term gain from Form 6781 line 8 (40% of section 1256 contracts) plus short-term gain recognized this year from earlier-year installment sales of investment property (Form 6252 line 26) and from earlier related-party exchanges disposed of within two years (Form 8824 line 24), plus short-term personal casualty or theft gain (Form 4684 line 15), as Schedule D line 4.",
    ),
    (
        "EXPLAIN_CAPITAL_GAIN_8A",
        "Totals for long-term transactions reported on a Form 1099-B or 1099-DA showing basis reported to the IRS, with no Form 8949 adjustment code and no wash-sale disallowance (Form 8949 box D or J, no adjustment).",
    ),
    (
        "EXPLAIN_CAPITAL_GAIN_8B",
        "Totals for long-term transactions reported on Form 8949 with box D or J checked: basis reported to the IRS, but with an adjustment code or a wash-sale disallowance.",
    ),
    (
        "EXPLAIN_CAPITAL_GAIN_N",
        "Totals for short-term transactions reported on Form 8949 with box B or H checked: a Form 1099-B or 1099-DA was received, but its basis was not reported to the IRS.",
    ),
    (
        "EXPLAIN_CAPITAL_GAIN_N_2",
        "Totals for short-term transactions with no Form 1099-B or 1099-DA received, including a reported sale of your main home and a deferral of gain invested in an opportunity fund (Form 8949 box C or I).",
    ),
    (
        "EXPLAIN_CAPITAL_GAIN_N_3",
        "Totals for long-term transactions reported on Form 8949 with box E or K checked: a Form 1099-B or 1099-DA was received, but its basis was not reported to the IRS.",
    ),
    (
        "EXPLAIN_CAPITAL_GAIN_N_4",
        "Totals for long-term transactions with no Form 1099-B or 1099-DA received, including a reported sale of your main home, a deferral of gain invested in an opportunity fund and previously deferred opportunity fund gain recognized (Form 8949 box F or L).",
    ),
    (
        "EXPLAIN_CARE_BENEFITS_EXCLUDED",
        "Employee benefits excluded after plan, expense and earned-income limits. Reduces the current care-credit expense ceiling.",
    ),
    (
        "EXPLAIN_CARE_BENEFITS_TAXABLE",
        "Additional taxable employee benefits reported on Form 1040 line 1e. Amounts already in W-2 box 1 are not added again.",
    ),
    (
        "EXPLAIN_CARE_CREDIT",
        "2026 care percentage and expense/earned-income limits; nonrefundable.",
    ),
    (
        "EXPLAIN_CARE_DISTRIBUTION_N_N_EXCEPTION",
        "Taxable distribution covered by the long-term-care exception. Ordinary pension income stays taxable. Spouse coverage needs a joint return.",
    ),
    (
        "EXPLAIN_CARE_DISTRIBUTION_N_N_LIMIT",
        "Annual plan ceiling: smaller of 10% of vested benefit and the 2026 $2,600 ceiling. Certified premiums and earlier qualifying payments further limit the exception.",
    ),
    (
        "EXPLAIN_CARE_DISTRIBUTION_N_N_USED",
        "Annual qualifying taxable amount used by this plan through this payment; multiple payments share its ceiling.",
    ),
    (
        "EXPLAIN_CHAPTER_ONE_NET_TAX",
        "Chapter1 tax after nonrefundable and refundable credits, including AMT, income-tax distribution additions, Form 8828 mortgage-subsidy recapture and education-credit recapture. Excludes payroll, self-employment, NIIT, section 4973 HSA excess and other Chapter43 excise taxes; used for the section 1341 restoration-year comparison, not the return's total tax.",
    ),
    (
        "EXPLAIN_CHARITY_CASH_DEDUCTION",
        "Current public-charity cash and eligible volunteer-service portion, taken first under the 2026 60%-of-AGI ceiling, then reduced by its share of the 0.5% floor before section 68. The floor separately charges appreciated stock first, then 30%-class student maintenance and whaling expenses, other noncash gifts and cash-class gifts.",
    ),
    (
        "EXPLAIN_CHARITY_CURRENT_CASH_N_CARRYFORWARD",
        "Only the current direct public-charity cash gift's 2026 60%-class percentage excess and any eligible share of the 2026 itemizer floor. The 2025 imported balance remains separate in its original contribution year; the appreciated-stock or student/whaling 30%-class carry is also shown separately. A 2026 balance can be used only in the five succeeding years, through 2031.",
    ),
    (
        "EXPLAIN_CHARITY_FLOOR_NOT_CARRIED",
        "Floor-disallowed amount not carried: absent current contribution-year percentage excess, no floor amount can carry. Even when a current-year gift has percentage excess, a prior-year cash, stock or household balance consumed in 2026 does not gain a new contribution year.",
    ),
    (
        "EXPLAIN_CHARITY_HOSTED_STUDENT_DEDUCTION",
        "Section 170(g) student-maintenance expense allowed under the 30%-of-AGI and residual 50%-of-AGI limits, after its 2026 0.5%-of-AGI floor share. Appreciated capital-gain property takes floor priority ahead of this class. This class is itemized only; any regular NOL limitation is reflected in the total charity deduction.",
    ),
    (
        "EXPLAIN_CHARITY_HOSTED_STUDENT_N_CARRYFORWARD",
        "Gross 2026 30%-class excess for student maintenance, plus its floor-disallowed share only when a percentage limit creates a carryover. It keeps its class and expires after 2031 unless used earlier; the total charity carryforward separately reflects any NOL reduction.",
    ),
    (
        "EXPLAIN_CHARITY_HOSTED_STUDENT_N_ELIGIBLE",
        "Section 170(g) actual student-hosting expense, at most $50 per qualifying student-month with at least 15 eligible days. It is a 30%-limit contribution for the use of the sponsoring organization, not a 60%-limit cash gift. Ineligible months, general household costs and reimbursement do not count.",
    ),
    (
        "EXPLAIN_CHARITY_NONCASH_DEDUCTION",
        "Personally purchased used goods, qualifying road vehicles, short-held or nonappreciated public stock and basis-elected appreciated public stock after the 50%-of-AGI limit and 2026 floor, which is charged to appreciated stock at FMV and student maintenance before this class. The source includes Form 8283 Section A or B facts when required, but does not produce the attachment.",
    ),
    (
        "EXPLAIN_CHARITY_NONCASH_N_CARRYFORWARD",
        "2026 supported noncash 50%-limit excess plus its floor-disallowed share only when any charitable percentage limit creates a carryover in the contribution year. This class keeps its 50% limit and expires after 2031 unless used earlier.",
    ),
    (
        "EXPLAIN_CHARITY_NONCASH_N_CARRYFORWARD_2",
        "Supported noncash 50%-limit excess retained from a year using the standard deduction, without any 0.5% itemizer floor. The five following years end in 2031.",
    ),
    (
        "EXPLAIN_CHARITY_N_CARRYFORWARD",
        "Current-year cash/service, supported noncash gifts, appreciated public stock and 30%-class student maintenance above their 2026 percentage limits retain their gift classes and year even when the standard deduction is selected. The standard-route cash deduction counts only qualifying cash gifts made to eligible organizations; the separately computed carryover keeps the hypothetical itemized percentage limits and excludes the itemizer-only 0.5% floor.",
    ),
    (
        "EXPLAIN_CHARITY_N_CARRYFORWARD_2",
        "Current-year 60%-class cash/service, 50%-class property, 30%-class public stock at FMV, and 30%-class student-maintenance and whaling percentage-limit excess, plus only the current-year floor share when such an excess exists. Retain each class and the 2026 year; the five following years end in 2031. After reducing any amount that increased retained regular NOLs. Imported cash and supported 2025 stock and household balances retain their original years; other imported classes require further worksheets.",
    ),
    (
        "EXPLAIN_CHARITY_N_NOL_REDUCTION",
        "Charitable excess removed because it increased regular NOL carryforward; the same amount cannot remain in both carryovers.",
    ),
    (
        "EXPLAIN_CHARITY_OTHER_N_PERCENT_DEDUCTION",
        "Combined section 170(g) student-maintenance and section 170(n) whaling contribution after the shared 30%-of-AGI and amended residual 50%-of-AGI limits and the 2026 itemizer floor. Both are current-year 30%-class contributions; the sources identify eligible amounts separately, but neither statute assigns the shared deduction between them. Any regular NOL limit appears in the total charity deduction.",
    ),
    (
        "EXPLAIN_CHARITY_OTHER_N_PERCENT_N_CARRYFORWARD",
        "Combined 2026 30%-class percentage-limit excess for hosted-student and statutory-capped whaling claims, plus the class's floor share only when a percentage limit creates a carryover. This shared class expires after 2031 unless used earlier; whaling costs beyond the separate $50,000-per-captain statutory cap do not carry.",
    ),
    (
        "EXPLAIN_CHARITY_PRIOR_CASH_CONSUMED_WITHOUT_DEDUCTION",
        "Prior-year carryover treated as paid within the 2026 percentage limit but not deducted because this return takes the standard deduction.",
    ),
    (
        "EXPLAIN_CHARITY_PRIOR_CASH_DEDUCTION",
        "Prior-year public-charity cash balance deducted on Schedule A line 14 (carryover from prior year) after the 2026 60% ceiling and 0.5%-of-AGI floor. A standard-deduction return claims none.",
    ),
    (
        "EXPLAIN_CHARITY_PRIOR_CASH_EXPIRED_AFTER_N",
        "Unused 2021 cash balance after the 2026 percentage-limit calculation expires rather than moving to 2027.",
    ),
    (
        "EXPLAIN_CHARITY_PRIOR_CASH_FLOOR_NOT_CARRIED",
        "The 2026 0.5%-of-AGI floor reduces the imported carryover's current deduction; it does not create a new contribution-year carryover for a 2021–2025 gift.",
    ),
    (
        "EXPLAIN_CHARITY_PRIOR_CASH_N_UNUSED",
        "Original-year balance left after 2026 percentage-limit use. A 2021 contribution expires after 2026; a later year's unused balance retains its original five-year period.",
    ),
    (
        "EXPLAIN_CHARITY_PRIOR_CASH_N_USED",
        "This original-year cash balance uses 2026 room oldest first after current-year public-charity gifts. A supported ordinary goods or vehicle gift reduces its aggregate 60%-of-AGI room; in the appreciated-stock pairing, current cash and then this balance use the 60% room before stock uses the remaining 50% room and separate 30% ceiling. With a hosted-student or whaling claim, current direct cash uses the 60% room first, then this older balance, then the 30%-class claim; that class's residual 50% limit restores up to 10% of AGI of the total cash. The balance is treated as paid even in a standard-deduction year; treatment as paid alone is not a tax deduction.",
    ),
    (
        "EXPLAIN_CHARITY_PRIOR_CASH_REMAINING_N",
        "Unused 2022–2025 original-year cash balances available for a later return, subject to each original five-year expiry and intervening-use rules.",
    ),
    (
        "EXPLAIN_CHARITY_PRIOR_HOUSEHOLD_N_CONSUMED_WITHOUT_DEDUCTION",
        "A percentage-usable prior noncash balance is treated as paid, but not deductible, in a standard-deduction year.",
    ),
    (
        "EXPLAIN_CHARITY_PRIOR_HOUSEHOLD_N_DEDUCTION",
        "Imported 2025 household-furnishing balance included once in 2026 Schedule A line 14 (carryover from prior year) after percentage limits and the itemizer floor; no itemized deduction is claimed on the standard route.",
    ),
    (
        "EXPLAIN_CHARITY_PRIOR_HOUSEHOLD_N_FLOOR_NOT_CARRIED",
        "The 2026 itemizer floor reduces this imported 2025 deduction; it cannot turn the older gift into a new 2026 carryover.",
    ),
    (
        "EXPLAIN_CHARITY_PRIOR_HOUSEHOLD_N_OPENING",
        "Reconciled unused 2025 50%-class balance entering 2026; it is not a current-year gift.",
    ),
    (
        "EXPLAIN_CHARITY_PRIOR_HOUSEHOLD_N_REMAINING_N",
        "Unused 2025 50%-class balance available in 2027, subject to intervening use; the five following years end after 2030, not after 2031.",
    ),
    (
        "EXPLAIN_CHARITY_PRIOR_HOUSEHOLD_N_USED",
        "2025 balance treated as paid after 2026 cash, 50%-class property and allowable 30%-class public stock use the 50%-of-AGI room. This older 50%-class balance then precedes a 2025 30%-class stock carry. Treatment as paid alone is not a tax deduction.",
    ),
    (
        "EXPLAIN_CHARITY_PRIOR_STOCK_N_CONSUMED_WITHOUT_DEDUCTION",
        "Prior 2025 balance treated as paid within the 2026 percentage limits but not deducted because this return takes the standard deduction.",
    ),
    (
        "EXPLAIN_CHARITY_PRIOR_STOCK_N_DEDUCTION",
        "Prior 2025 public-stock balance included once in the 2026 Schedule A line 14 carryover from prior year, after the current-year percentage ceilings and itemizer floor. A standard-deduction return claims none.",
    ),
    (
        "EXPLAIN_CHARITY_PRIOR_STOCK_N_FLOOR_NOT_CARRIED",
        "The 2026 itemizer floor reduces the imported 2025 balance's deduction and cannot create a new 2026 contribution-year carryover.",
    ),
    (
        "EXPLAIN_CHARITY_PRIOR_STOCK_N_OPENING",
        "Reconciled unused 2025 30%-class FMV balance entering 2026; original donation is not entered again as a 2026 gift.",
    ),
    (
        "EXPLAIN_CHARITY_PRIOR_STOCK_N_ORIGINAL_BASIS",
        "Original adjusted basis retained with the 2025 Form 8283 Section A facts and any future capital-gain-property election review; the 2026 FMV carry path does not use basis as its claim.",
    ),
    (
        "EXPLAIN_CHARITY_PRIOR_STOCK_N_ORIGINAL_VALUE",
        "Original 2025 public-market fair value of the directly donated long-term stock lot. This is Form 8283 Section A evidence, not a second contribution or 2026 deduction.",
    ),
    (
        "EXPLAIN_CHARITY_PRIOR_STOCK_N_REMAINING_N",
        "Unused original-year 30%-class balance still available in 2027, subject to intervening use; its five following carry years are 2026 through 2030, not 2026 through 2031.",
    ),
    (
        "EXPLAIN_CHARITY_PRIOR_STOCK_N_USED",
        "2025 balance treated as paid in 2026 after current-year stock gifts use the 30%-class and residual public-charity 50% limits. Treatment as paid alone does not mean a tax deduction.",
    ),
    (
        "EXPLAIN_CHARITY_PUBLIC_STOCK_N_CARRYFORWARD",
        "Gross 2026 30%-class FMV stock excess, plus its floor share only when a percentage ceiling creates a carryover. Retain this class through 2031; the total carryforward separately reflects any NOL reduction.",
    ),
    (
        "EXPLAIN_CHARITY_PUBLIC_STOCK_N_CLAIM",
        "Purchased, section 1014 inherited or ordinary section 1015 gifted publicly traded stock delivered to an eligible public charity: short-held or nonappreciated shares claim the lesser of FMV and basis in the 50% class; appreciated long-term shares claim FMV in the 30% class or basis under the annual reduction election. The gift realizes no sale gain; Form 8283 Section A applies when total noncash gifts exceed $500.",
    ),
    (
        "EXPLAIN_CHARITY_PUBLIC_STOCK_N_PERCENT_DEDUCTION",
        "Appreciated public stock at FMV after the separate 30%-of-AGI and remaining public-charity 50%-of-AGI ceilings. The 2026 itemizer floor charges this class first. No immediate standard-route deduction; the total charity deduction separately reflects any NOL limit.",
    ),
    (
        "EXPLAIN_CHARITY_VOLUNTEER_N_EXPENSE",
        "Unreimbursed direct volunteer service expense: qualifying car, away travel, uniform, foster care, selected-youth outing or permanent diaconate cost. This enters the cash-contribution class once; donated time and personal costs are not deductible. Retain reliable written records and any required charity acknowledgment.",
    ),
    (
        "EXPLAIN_CHARITY_WHALING_DEDUCTION",
        "Section 170(n) statutory-capped 2026 whaling expenses after the 30%-of-AGI other-contribution limit, residual 50% ceiling and 0.5%-of-AGI itemizer floor. Any overall NOL limit appears in the total charity deduction.",
    ),
    (
        "EXPLAIN_CHARITY_WHALING_ELIGIBLE_EXPENSES",
        "Unreimbursed, substantiated 2026 reasonable and necessary sanctioned whaling expenses after allocating any boat also used for other activities by sanctioned-use days over 365 days. This is before each captain's separate $50,000 statutory cap or AGI percentage limits.",
    ),
    (
        "EXPLAIN_CHARITY_WHALING_N_CARRYFORWARD",
        "Only percentage-limit excess of the statutory-capped 2026 whaling contribution, plus its eligible floor share when a percentage ceiling creates a carryover. Expense above the separate $50,000 annual statutory cap never carries; this 30% class expires after 2031 unless used earlier.",
    ),
    (
        "EXPLAIN_CHARITY_WHALING_N_ELIGIBLE_EXPENSES",
        "This captain's eligible whaling expenses after any mixed-use boat allocation, before that individual's $50,000 statutory cap.",
    ),
    (
        "EXPLAIN_CHARITY_WHALING_N_STATUTORY_CLAIM",
        "The 2026 section 170(n) $50,000 annual cap applies to this recognized captain separately, including on a joint return.",
    ),
    (
        "EXPLAIN_CHARITY_WHALING_STATUTORY_CLAIM",
        "Section 170(n) deems up to $50,000 of each recognized captain's 2026 expenses a charitable contribution. It is itemizer-only and enters the 30%-of-AGI other-contribution class, not the 60% public-charity cash or nonitemizer class. Other charity limits apply afterward.",
    ),
    (
        "EXPLAIN_CHILD_ANNUITY_N_N",
        "Temporary child-annuity cost recovery is limited to monthly payments and remaining cost; unused exclusion remains separate from investment and is usable only against eligible later payments. Ending the annuity does not automatically create a deduction.",
    ),
    (
        "EXPLAIN_CHILD_CREDIT",
        "Child and other-dependent credits after MAGI phaseout and prior nonrefundable credits.",
    ),
    (
        "EXPLAIN_CIVIL_SERVICE_DEATH_N_N",
        "Civil-service lump sum plus earlier tax-free annuity recovery, less employee cost, determines taxable income. Any remaining cost is not an automatic deduction on this beneficiary's return.",
    ),
    (
        "EXPLAIN_CLAIM_REPAYMENT",
        "Restoration-year Chapter1 tax less refundable credits, without the repayment deduction or credit. Refigured capital losses and minimum-tax credits affect this comparison only; payroll, NIIT and excise taxes are excluded. Saved source history and actual current income remain unchanged.",
    ),
    (
        "EXPLAIN_CLEAN_VEHICLE_MODIFIED_AGI_N",
        "Form 8936 line 2: 2026 AGI plus excluded Puerto Rico income, Form 2555 lines 45 and 50, and Form 4563 line 15.",
    ),
    (
        "EXPLAIN_CLEAN_VEHICLE_MODIFIED_AGI_N_2",
        "Form 8936 line 4: prior-year modified AGI tested against the 2025 filing-status limit.",
    ),
    (
        "EXPLAIN_CLEAN_VEHICLE_N_POTENTIAL",
        "2026 Schedule A (Form 8936) for VIN {0}: eligible personal-use credit before the Form 1040 tax limit.",
    ),
    (
        "EXPLAIN_COOPERATIVE_N_N_DISTRIBUTIONS",
        "Total cooperative distributions (Form 1099-PATR boxes 1, 2, 3 and 5), routed to this owner's farm for Schedule F line 3a.",
    ),
    (
        "EXPLAIN_COOPERATIVE_N_N_NONQUALIFIED",
        "Taxable distributions above the box 8 qualified items, excluded from the farm's qualified business income.",
    ),
    (
        "EXPLAIN_COOPERATIVE_N_N_PATRON_REDUCTION",
        "Form 8995-A Schedule D patron reduction on the box 7 qualified payments: zero because the farm has no W-2 wages allocable to them.",
    ),
    (
        "EXPLAIN_COOPERATIVE_N_N_TAXABLE",
        "Taxable distributions after the personal-purchase dividends, routed to this owner's farm for Schedule F line 3b, self-employment earnings and QBI.",
    ),
    (
        "EXPLAIN_CURRENT_NET_OPERATING_LOSS",
        "Current-year NOL after capital-loss and nonbusiness-deduction limits; the separate Form 461 carryover is added below.",
    ),
    (
        "EXPLAIN_DC_HOMEBUYER_CARRYFORWARD_N",
        "2026 Form 8859 line 4: unused balance carried to 2027 without a fixed expiration.",
    ),
    (
        "EXPLAIN_DC_HOMEBUYER_CREDIT",
        "2026 Form 8859 lines 1–3: prior DC first-time homebuyer carryforward, limited by tax after the listed credits.",
    ),
    (
        "EXPLAIN_DEBT_CANCELLATION_N_TAXABLE",
        "Personal recourse debt cancellation, including nondeductible interest, is ordinary Schedule 1 line 8c income unless a confirmed exception applies. A genuine gift or bequest is excluded without reducing tax attributes. Not wages, self-employment earnings or investment income.",
    ),
    (
        "EXPLAIN_DEBT_N_0BASIS_REDUCTION",
        "Excluded debt applied to property basis on January 1, 2027 in the Treas. Reg. 1.1017-1 order, limited for a bankruptcy or insolvency by post-discharge basis plus cash minus liabilities. No current tax or deduction change.",
    ),
    (
        "EXPLAIN_DEBT_N_0PROPERTY_N_N",
        "{0}: January 1, 2027 basis reduction in proportion to adjusted basis within its property category. Retain the reduced basis and the reduction for later section 1245 or 1250 ordinary-income recapture; this is not current depreciation.",
    ),
    (
        "EXPLAIN_DEBT_N_FORM982_LINE",
        "Box checked: the part excluded because of the title 11 case or insolvency.",
    ),
    ("EXPLAIN_DEBT_N_FORM982_LINE2", "Total discharged debt excluded from gross income."),
    (
        "EXPLAIN_DEBT_N_FORM982_LINE_10A",
        "Applied to the basis of property held on January 1, 2027, secured business property first and personal-use property last.",
    ),
    (
        "EXPLAIN_DEBT_N_FORM982_LINE_11A",
        "Farm debt applied to depreciable qualified property not reduced on line 5.",
    ),
    ("EXPLAIN_DEBT_N_FORM982_LINE_11B", "Farm debt applied to land used in the farming business."),
    ("EXPLAIN_DEBT_N_FORM982_LINE_11C", "Farm debt applied to other qualified property."),
    ("EXPLAIN_DEBT_N_FORM982_LINE_1C", "Box checked: the qualified farm debt excluded."),
    (
        "EXPLAIN_DEBT_N_FORM982_LINE_1D",
        "Box checked: the qualified real property business debt excluded.",
    ),
    (
        "EXPLAIN_DEBT_N_FORM982_LINE_N",
        "Qualified real property business debt applied to the basis of depreciable real property, the securing property first.",
    ),
    (
        "EXPLAIN_DEBT_N_FORM982_LINE_N_2",
        "Excluded debt you elected to apply first to the basis of depreciable property.",
    ),
    (
        "EXPLAIN_DEBT_N_FORM982_LINE_N_3",
        "Applied to the 2026 NOL, then NOL carryovers to 2026 in the order of their years.",
    ),
    (
        "EXPLAIN_DEBT_N_FORM982_LINE_N_4",
        "Applied to the general business credit carryover, 33 1/3 cents of credit for each dollar.",
    ),
    (
        "EXPLAIN_DEBT_N_FORM982_LINE_N_5",
        "Applied to the minimum tax credit available at the beginning of 2027, 33 1/3 cents for each dollar.",
    ),
    (
        "EXPLAIN_DEBT_N_FORM982_LINE_N_6",
        "Applied to the 2026 net capital loss and capital loss carryovers.",
    ),
    ("EXPLAIN_DEBT_N_FORM982_LINE_N_7", "Applied to passive activity loss carryovers from 2026."),
    (
        "EXPLAIN_DEBT_N_FORM982_LINE_N_8",
        "Applied to the foreign tax credit carryover, 33 1/3 cents for each dollar.",
    ),
    (
        "EXPLAIN_DEBT_N_N",
        "Seller purchase-price adjustment: reduce the identified property's basis immediately, without cancellation income or a current deduction. Retain the adjusted basis for later dispositions; this is not a Form 982 next-year attribute reduction.",
    ),
    (
        "EXPLAIN_DEBT_N_N_2",
        "Form 982 box 1a and line 2: debt discharged in a confirmed chapter 12 or 13 case is excluded without a solvency limit. Attribute reduction follows the current-year income tax calculation.",
    ),
    (
        "EXPLAIN_DEBT_N_N_3",
        "Form 982 line 2: discharged debt limited to liabilities exceeding all asset fair market value immediately before discharge. The balance is taxable.",
    ),
    ("EXPLAIN_DEBT_N_N_AFTER", "Balance carried to 2027 after the debt reduction."),
    (
        "EXPLAIN_DEBT_N_N_CAPITAL_REDUCTION",
        "Capital loss carryforward reduced dollar for dollar; the carryforward to 2027 keeps the balance shown after reduction.",
    ),
    (
        "EXPLAIN_DEBT_N_N_ELECTION_BASIS_REDUCTION",
        "Excluded debt applied first to depreciable property basis by the section 108(b)(5) election.",
    ),
    (
        "EXPLAIN_DEBT_N_N_FARM_BASIS_REDUCTION",
        "Farm debt applied to the basis of qualified property: depreciable property, then farm land, then other.",
    ),
    (
        "EXPLAIN_DEBT_N_N_FARM_EXCLUSION",
        "Form 982 box 1c: qualified farm debt above any insolvency exclusion, within adjusted tax attributes plus the basis of qualified property.",
    ),
    (
        "EXPLAIN_DEBT_N_N_FTC_REDUCTION",
        "Foreign tax credit carryover reduced by 33 1/3 cents for each dollar of excluded debt applied.",
    ),
    (
        "EXPLAIN_DEBT_N_N_NOL_REDUCTION",
        "Separate regular/AMT loss reduction after current tax: current-year NOL first, then unused prior losses in original-year order, before basis. Current tax and deductions remain fixed.",
    ),
    (
        "EXPLAIN_DEBT_N_N_PASSIVE_REDUCTION",
        "Passive activity loss carryover reduced dollar for dollar.",
    ),
    (
        "EXPLAIN_DEBT_N_N_REAL_PROPERTY_BASIS_REDUCTION",
        "Qualified real property business debt applied to the basis of depreciable real property.",
    ),
    (
        "EXPLAIN_DEBT_N_N_REAL_PROPERTY_EXCLUSION",
        "Form 982 box 1d: qualified real property business debt above any insolvency exclusion, within the principal over the net value of the securing property and the basis of depreciable real property.",
    ),
    (
        "EXPLAIN_DEBT_N_N_REDUCTION_GBC",
        "General business credit carryover reduced by 33 1/3 cents for each dollar of excluded debt applied; the balance carries to 2027.",
    ),
    (
        "EXPLAIN_DEBT_N_N_REDUCTION_MTC",
        "Minimum tax credit available at the beginning of 2027 reduced by 33 1/3 cents for each dollar of excluded debt applied.",
    ),
    (
        "EXPLAIN_DEBT_N_SALE_GAIN",
        "Form 8949 sale: a later sale using basis after the seller's price adjustment, or property the creditor took at its amount realized. Personal losses are disallowed; taxable gains retain their holding period and collectible treatment.",
    ),
    (
        "EXPLAIN_DEBT_N_STUDENT_EXCLUSION",
        "Qualifying student-loan cancellation exception: excluded without Form 982 tax-attribute reduction. Forgiven interest is not a student-loan interest payment. The expired general 2021–2025 exclusion is not used.",
    ),
    (
        "EXPLAIN_DEDUCTION",
        "Itemized medical, capped SALT, mortgage interest and points, cash gifts after the 0.5% AGI floor and remaining qualifying educator expenses and limited gambling losses.",
    ),
    (
        "EXPLAIN_DEDUCTION_2",
        "Standard deduction including age/blindness additions and eligible non-itemizer cash gifts.",
    ),
    (
        "EXPLAIN_DISASTER_MITIGATION_N_EXCLUDED",
        "2026 section 139(g) hazard-mitigation payment excluded from income. The excluded amount cannot increase property basis or fund a second deduction or credit.",
    ),
    (
        "EXPLAIN_DISASTER_RELIEF_N_EXCLUDED",
        "2026 section 139 qualified disaster relief excluded from gross income. No deduction or credit may be claimed for an expenditure paid by the excluded amount.",
    ),
    (
        "EXPLAIN_DISASTER_RELIEF_N_WITHHOLDING",
        "Federal income tax actually withheld on a payer's Form 1099 for an excluded section 139 payment. Claim the withheld tax on Form 1040 line 25b; the payment remains excluded from AGI.",
    ),
    (
        "EXPLAIN_DISTRIBUTION_N_N_BASIS_REDUCTION",
        "Return of capital applied against this lot's basis; not taxable income or a loss.",
    ),
    (
        "EXPLAIN_DISTRIBUTION_N_N_CAPITAL_GAIN",
        "Gain beyond this lot's remaining basis, or loss on final liquidation and cancellation. Holding period determines capital treatment; interim payments never create a loss.",
    ),
    (
        "EXPLAIN_DISTRIBUTION_N_N_REMAINING_BASIS",
        "Basis remaining after this distribution; zero when final liquidation cancels the shares. Retain for later distributions or sales, without duplicating an adjustment already in broker records.",
    ),
    (
        "EXPLAIN_DISTRIBUTION_N_PROPERTY_N_BASIS",
        "{0}: basis of the property received is its distribution-date fair market value. Retain this allocation with the asset records for a later sale.",
    ),
    (
        "EXPLAIN_DOMESTIC_ABUSE_USED_TAXPAYER",
        "Qualified domestic-abuse gross amount used through 2026, including prior-year claims. Retain for later returns.",
    ),
    (
        "EXPLAIN_DONATED_VEHICLE_N_CLAIM",
        "Personally used road vehicle: an ordinary unrelated sale claims the smaller of supported fair market value and proceeds when proceeds exceed $500, or the smaller of fair market value and $500 otherwise. A certified significant-use, material-improvement or needy-person exception claims fair market value; when the similar road-vehicle group exceeds $5,000, its FMV-exception vehicles need qualified appraisals and Section B evidence. Gross-proceeds-limited vehicles stay in Section A. Retain the timely charity acknowledgment and attach Form 8283 when required. This on-screen estimate does not prepare the attachment.",
    ),
    (
        "EXPLAIN_EARLY_DISTRIBUTION_TAX",
        "10% additional tax, including taxable Roth earnings and recent taxable-conversion recapture, or 25% for early traditional SIMPLE IRA distributions within two years, after supported source-based exceptions.",
    ),
    (
        "EXPLAIN_EARLY_FILER",
        "Separate filer Form 5329 Part I amount, rounded after aggregating source amounts.",
    ),
    (
        "EXPLAIN_EARNED_INCOME_CREDIT",
        "EIC is unavailable because a filer is another taxpayer’s EIC qualifying child.",
    ),
    (
        "EXPLAIN_EARNED_INCOME_CREDIT_2",
        "Draft 2026 IRS EIC Table: look up earned income, then AGI when required, and use the lower amount. Reconcile with the final 2026 table before filing use.",
    ),
    ("EXPLAIN_EDUCATION_CREDIT", "Nonrefundable education credits, subject to available tax."),
    (
        "EXPLAIN_EDUCATION_CREDIT_RECAPTURE_TAX",
        "Form 1040 line 16 box 3 (enter ECR): tax-liability increase after refiguring the sole fully usable 2025 education credit for post-filing refunds or tax-free assistance. No 2026 education expense is reduced here. It is part of line 18, the base of the credit limits.",
    ),
    (
        "EXPLAIN_EIC_INVESTMENT_INCOME",
        "Publication 596 Worksheet 1 investment income, excluding section 1231 capital from the capital line and counting passive property income once in the passive line.",
    ),
    (
        "EXPLAIN_ELDERLY_CREDIT",
        "Schedule R credit for age-qualified filers and supported disability retirement income, limited by disability income where applicable, exempt benefits, AGI and available tax.",
    ),
    (
        "EXPLAIN_EMPLOYEE_STOCK_N_N",
        "Ordinary employee stock source records determine compensation and sale basis. Compensation was already reported as wages in the recognition year and is not added again. Regular and AMT sale basis are the same. For an elected-stock forfeiture, basis excludes previously taxed compensation.",
    ),
    (
        "EXPLAIN_EMPLOYEE_TIPS_N_N_TAX",
        "Employee share of unreported-tip payroll tax from this filer's Form 4137; separate from withholding and self-employment tax.",
    ),
    (
        "EXPLAIN_EMPLOYEE_TIP_INCOME",
        "Additional tip wages outside W-2 box 1, including noncash tips and cash tips below the monthly payroll threshold.",
    ),
    (
        "EXPLAIN_EMPLOYEE_WAGES_N_N_TAX",
        "Form 8919 employee payroll tax after W-2 wages and covered unreported tips; not employer withholding or self-employment tax.",
    ),
    (
        "EXPLAIN_EMPLOYER_LOAN_INTEREST_REMOVED",
        "Interest your employer paid tax-free under an educational assistance program, removed from the student-loan interest claim before the $2,500 cap and phase-out; it cannot also be deducted.",
    ),
    (
        "EXPLAIN_ESPP_SALE_N_N",
        "Section 423 holding periods select grant-discount or exercise-discount compensation. Basis includes compensation; reported wages are not added again. Unreported compensation enters Schedule 1 line 8k.",
    ),
    (
        "EXPLAIN_ESTATE_PAID_MEDICAL_ELECTION",
        "Elected estate-paid expenses incurred in 2026; included in medical expenses before the 7.5% AGI floor. Attach the required statement that these amounts were not claimed for estate tax and waive the estate-tax deduction. This App does not submit the election.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_67E_DEDUCTION",
        "Allocated terminating-estate or trust section 67(e) excess deduction on Schedule 1 line 24k, combined across K-1 sources before rounding. It reduces AGI but does not create a second carryover of the fiduciary expense.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_IRD_ESTATE_TAX_DEDUCTION",
        "Schedule A line 17e section 691(c) estate-tax deduction allocated solely to box 1 taxable IRD interest. Does not reduce AGI; the allowed itemized share also reduces Form 8960 net investment income.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_11A_67E_DEDUCTION",
        "Final-year Form 1041 K-1 box 11 code A section 67(e) deduction allocated to the beneficiary succeeding to property. Enters Schedule 1 line 24k once, reduces AGI, and cannot be carried to another beneficiary tax year.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_11B_PERSONAL_PROPERTY_TAX",
        "Beneficiary-allocated final-year personal property tax retains its non-miscellaneous itemized character and joins the beneficiary's SALT limit; unused excess does not carry to another year.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_11B_REAL_ESTATE_TAX",
        "Final-year box 11 code B noninvestment real-estate tax allocated to the beneficiary, included on Schedule A line 5b and subject to the beneficiary's SALT limit; not a Form 8960 deduction or AMT deduction.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_11B_STATE_INCOME_TAX",
        "Final-year box 11 code B state/local income tax already limited by the fiduciary, available on Schedule A line 5a only when the beneficiary elects income rather than sales tax and subject to the beneficiary's SALT limit; no Form 8960 allocation.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_11B_STATE_SALES_TAX",
        "Final-year box 11 code B general sales tax already limited by the fiduciary, available on Schedule A line 5a only when the beneficiary elects general sales rather than income tax and subject to the beneficiary's SALT limit; no Form 8960 allocation.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_12A_AMT_ADJUSTMENT",
        "Form 1041 K-1 box 12 code A total on Form 6251 line 2j; regular AGI is unchanged. Codes B through F are portions, not another addition to AMTI.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_12B_AMT_QUALIFIED_DIVIDENDS",
        "Form 1041 K-1 box 12 code B is entirely included in code A, and changes only the qualified-dividend amount used in the Form 6251 preferential-rate calculation.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_12C_AMT_SHORT_GAIN",
        "Form 1041 K-1 box 12 code C is entirely included in code A. It refigures AMT Schedule D line 5; regular Schedule D and AGI are unchanged, and code C is not added to AMTI again.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_12D_AMT_LONG_GAIN",
        "Form 1041 K-1 box 12 code D is entirely included in code A. It refigures AMT Schedule D line 12; regular Schedule D and AGI are unchanged, and code D is not added to AMTI again.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_12E_AMT_UNRECAPTURED_GAIN",
        "Form 1041 K-1 box 12 code E is a section 1250 portion included in code D and code A, not another AMTI addition. It refigures line 11 of the AMT Unrecaptured Section 1250 Gain Worksheet; code D refigures AMT Schedule D line 12. Neither changes regular Schedule D or AGI.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_12F_AMT_RATE_N_GAIN",
        "Form 1041 K-1 box 12 code F is the 28%-rate portion already included in D and A. It refigures line 4 of the AMT 28% Rate Gain Worksheet; regular Schedule D and AGI are unchanged, and it is not added to AMTI again.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_14A_PRIVATE_ACTIVITY",
        "Specified private-activity interest included in tax-exempt box 14 code A; added once to the shared Form 6251 line 2g preference, not to regular AGI.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_14H_NIIT_LINE_N",
        "Form 1041 K-1 box 14 code H adjustment on Form 8960 line 7, separate from the portfolio income boxes.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_14H_NIIT_MAGI",
        "Corresponding Form 8960 modified-AGI adjustment from the fiduciary's box 14 code H statement.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_14I_SECTION199A_DIVIDENDS",
        "Beneficiary's fiduciary-allocated section 199A dividends are already included in box 2a ordinary dividends; qualified amounts join the shared REIT/PTP deduction calculation without adding income again.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_LINE33E_ROYALTY_DEDUCTIONS",
        "Fiduciary-apportioned box 9 royalty deductions on Schedule E line 33 column (e), separate from gross royalties in column (f). These deductions reduce income once.",
    ),
    (
        "LINE_ESTATE_TRUST_N_LINE33E_ROYALTY_DEDUCTIONS",
        "Estate trust {0} line 33e royalty deductions",
    ),
    (
        "EXPLAIN_SCHEDULE_E_ESTATE_TOTALS",
        "Schedule E Part III totals use the actual allowed passive and nonpassive income and deduction columns. Line 37 combines line 35 income and line 36 losses; no income or deduction is added again.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_LINE33F_ROYALTIES",
        "Schedule K-1 (Form 1041) box 5 gross royalties: Schedule E line 33, column (f). Directly apportioned royalty deductions are separate in column (e), not netted from this income column.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_9A_ROYALTY_DEPRECIATION",
        "Fiduciary-apportioned box 9 code A straight-line depreciation of one box 5 nonbusiness sound-recording royalty asset, deducted once on Schedule E line 33(e). The confirmed same-character amount reduces AMT income and net investment income through the shared royalty calculation.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_9B_ROYALTY_DEPLETION",
        "Fiduciary-apportioned box 9 code B cost depletion from one box 5 nonbusiness mineral royalty property, deducted once on Schedule E line 33(e). The confirmed same-character amount also reduces AMT income and net investment income through the shared royalty calculation.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_9C_ROYALTY_AMORTIZATION",
        "Fiduciary-apportioned box 9 code C amortization from the same box 5 nonbusiness royalty activity, deducted once on Schedule E line 33(e). The confirmed same-character amount also reduces AMT income and net investment income through the shared royalty calculation.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_ALLOCATED_ESTIMATED_TAX",
        "Form 1041 K-1 box 13 code A estimated tax allocated on timely Form 1041-T, included once on Form 1040 line 26. For Form 2210 it is treated as paid January 15, 2027.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_BACKUP_WITHHOLDING",
        "Form 1041 K-1 box 13 code B backup withholding, counted once on Form 1040 line 25c, not subtracted from income or capital gain.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_N",
        "Fiduciary-allocated successor NOL box total, with each original estate/trust loss year preserved; regular and AMT deductions and carryforwards are calculated separately.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_N_14F_GROSS_FARMING_FISHING_INCOME",
        "Form 1041 K-1 box 14 code F gross farming and fishing income included in box 6, shown on Schedule E line 42 for the Form 2210 farmer and fisherman two-thirds test; it adds no income.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_N_2",
        "Beneficiary-share Form 1041 K-1 amount carried to the corresponding Form 1040, Schedule D or Schedule E line; qualified dividends and special long-term gain classes are included in, not added to, their parent totals. Royalty-only box 5 enters Schedule E line 33(f).",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_N_3",
        "Final-year Form 1041 K-1 box 11 capital loss carryover allocated to the beneficiary succeeding to property; short-term code C reaches Schedule D line 5 and long-term code D reaches line 12, with unused losses retained by character.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_N_ESTATE_TAX",
        "Fiduciary-allocated section 691(c) estate-tax deduction attributable solely to taxable box 1 IRD interest; claim on Schedule A line 17e only when itemizing.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_N_N_BUSINESS_APPORTIONED_DEDUCTIONS",
        "Form 1041 K-1 box 9 codes A–C directly apportioned to the box 6 activity, deducted once against that activity on Schedule E line 33 column (c) or (e); the same amount applies for AMT and net investment income.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_N_N_BUSINESS_INCOME",
        "Form 1041 K-1 box 6 ordinary business income in which you materially participated: nonpassive, Schedule E line 33 column (f), outside Form 8582 and reversed out of net investment income on Form 8960 line 4b.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_N_N_BUSINESS_INCOME_2",
        "Form 1041 K-1 box 6 ordinary business income without material participation: passive, Schedule E line 33 column (d), available on Form 8582 against other passive losses and included in net investment income.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_N_N_OTHER_RENTAL_APPORTIONED_DEDUCTIONS",
        "Form 1041 K-1 box 9 codes A–C directly apportioned to the box 8 activity, deducted once against that activity on Schedule E line 33 column (c); the same amount applies for AMT and net investment income.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_N_N_OTHER_RENTAL_INCOME",
        "Form 1041 K-1 box 8 other rental income: passive, Schedule E line 33 column (d), available on Form 8582 against other passive losses and included in net investment income.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_N_N_RENTAL_REAL_ESTATE_APPORTIONED_DEDUCTIONS",
        "Form 1041 K-1 box 9 codes A–C directly apportioned to the box 7 activity, deducted once against that activity on Schedule E line 33 column (c); the same amount applies for AMT and net investment income.",
    ),
    (
        "EXPLAIN_ESTATE_TRUST_N_N_N_RENTAL_REAL_ESTATE_INCOME",
        "Form 1041 K-1 box 7 net rental real estate income: passive, Schedule E line 33 column (d), available on Form 8582 against other passive losses (no special allowance applies to income) and included in net investment income.",
    ),
    (
        "EXPLAIN_EXCESS_BUSINESS_LOSS",
        "2026 Form 461 adds back net business loss beyond $256,000 ($512,000 joint); employee wages do not increase that limit. The disallowed amount carries forward as an NOL.",
    ),
    (
        "EXPLAIN_EXCESS_BUSINESS_LOSS_CARRYFORWARD",
        "Form 461 disallowed loss carried to the following year, shown separately from the current-year Form 172 NOL. Included once in the total NOL carryforward below.",
    ),
    (
        "EXPLAIN_FAMILY_LEAVE_1099G",
        "Paid family leave benefits (Form 1099-G box 10) on Schedule 1 line 8z under Rev. Rul. 2025-4; only the includible family-leave portion, with no medical-leave amount present. Not wages, self-employment earnings, EIC or IRA compensation.",
    ),
    (
        "EXPLAIN_FARMING_FISHING_GROSS_INCOME_N",
        "Gross income from farming or fishing for the farmer and fisherman estimated-tax test: Schedule F line 9, Form 1041 K-1 box 14 code F, Form 1065 K-1 box 14 code B and commercial fishing Schedule C line 7.",
    ),
    (
        "EXPLAIN_FARM_AVERAGING",
        "Schedule J ordinary-farm election. The current-year reduced-income tax uses the applicable 2026 Tax Table or special worksheet; base-year recomputation uses the IRS rate schedules, not filing Tax Table intervals.",
    ),
    (
        "EXPLAIN_FARM_AVERAGING_N_N",
        "Retained base-year income/tax and rate-schedule recomputation after one-third of elected ordinary farm income. Prior returns and carryovers are not amended by this allocation.",
    ),
    (
        "EXPLAIN_FARM_AVERAGING_QBI_DEDUCTION",
        "Farm-attributable QBI deduction from the same activity calculation after loss netting and wage/property limits. REIT and nonfarm components are not subtracted from eligible farm income. Mixed overall-income-cap and minimum-deduction attribution remains unresolved.",
    ),
    (
        "EXPLAIN_FARM_INSTALLMENT_AVERAGING_ELECTED_INCOME",
        "The selected ordinary farm election limited to Form 1062 reduced taxable income for this comparison only; no new actual election is made.",
    ),
    (
        "EXPLAIN_FARM_INSTALLMENT_AVERAGING_N_N",
        "Form 1062 comparison only: Schedule J allocation after its reduced taxable-income ceiling. Actual election and retained base-year records are unchanged.",
    ),
    (
        "EXPLAIN_FARM_INSTALLMENT_N_ACTUAL_CREDITS",
        "Form 1062 applicable foreign and personal credits, including personal credits against AMT. Section 53 minimum-tax credit is excluded from this comparison and remains on the actual return.",
    ),
    (
        "EXPLAIN_FARM_INSTALLMENT_N_ACTUAL_FOREIGN_CREDIT",
        "Form 1062 line 4a: the actual payee-statement foreign tax credit, before personal credits.",
    ),
    (
        "EXPLAIN_FARM_INSTALLMENT_N_ACTUAL_REGULAR_LIABILITY",
        "Form 1062 line 3: regular income tax plus premium-credit repayment, before applicable credits.",
    ),
    (
        "EXPLAIN_FARM_INSTALLMENT_N_APPLICABLE",
        "Form 1062 line 14: tax attributable to all elected sales; this is a deferral offset, not withholding or a payment made.",
    ),
    (
        "EXPLAIN_FARM_INSTALLMENT_N_CURRENT",
        "Annual tax less deferred liability, before other payments and refundable credits. A negative position contributes to the estimated overpayment; annual tax and future installments remain unchanged.",
    ),
    (
        "EXPLAIN_FARM_INSTALLMENT_N_ELECTED_GAIN",
        "Form 1062 line 7: all elected gains together, not independent per-sale tax differences.",
    ),
    (
        "EXPLAIN_FARM_INSTALLMENT_N_FIRST",
        "Form 1062 line 15: first 25% installment, rounded to a whole dollar, due without a filing extension.",
    ),
    (
        "EXPLAIN_FARM_INSTALLMENT_N_REDUCED_CREDITS",
        "Form 1062 line 12z: aggregate refigured foreign and personal credits cannot exceed reduced regular liability; eligibility and phaseout amounts are retained.",
    ),
    (
        "EXPLAIN_FARM_INSTALLMENT_N_REDUCED_FOREIGN_CREDIT",
        "Form 1062 line 12a: payee-statement credit re-limited to reduced regular tax; no election-year carryover.",
    ),
    (
        "EXPLAIN_FARM_INSTALLMENT_N_REDUCED_REGULAR_LIABILITY",
        "Form 1062 line 11: refigured regular income tax plus unchanged premium-credit repayment; credit limits use this liability.",
    ),
    (
        "EXPLAIN_FARM_INSTALLMENT_N_REDUCED_REGULAR_TAX",
        "Form 1062 line 9: shared tax worksheet using redetermined gain character and the comparison-specific capped Schedule J worksheet.",
    ),
    (
        "EXPLAIN_FARM_INSTALLMENT_N_REDUCED_TAXABLE",
        "Form 1062 line 8: actual taxable income less elected gains; AGI and deductions are unchanged.",
    ),
    (
        "EXPLAIN_FARM_INSTALLMENT_N_REGULAR_ADDITIONS",
        "Form 1062 lines 2 and 10: unchanged premium-credit repayment from actual household MAGI, coverage and advance payments.",
    ),
    (
        "EXPLAIN_FARM_INSTALLMENT_N_REMAINING",
        "Liability deferred after the first installment; remains owed and may accelerate on default or death.",
    ),
    (
        "EXPLAIN_FARM_N_ACCRUAL_SALES",
        "Sales recognized under the accrual method, including products raised and purchased.",
    ),
    (
        "EXPLAIN_FARM_N_CCC_ADVANCES",
        "Current advances included under section 77; prior taxed loans are not included again.",
    ),
    (
        "EXPLAIN_FARM_N_CCC_FORFEITURE",
        "Current-year forfeiture principal not previously taxed; earlier forfeitures are not taxed again.",
    ),
    (
        "EXPLAIN_FARM_N_CCC_FORFEITURE_REPORTED",
        "Full current forfeiture principal for Schedule F line 5b or 40b, including income-elected loans; only the separate taxable amount enters income.",
    ),
    (
        "EXPLAIN_FARM_N_CCC_INCOME",
        "CCC income before the separate paid-deficiency expense enters farm receipts once.",
    ),
    (
        "EXPLAIN_FARM_N_CCC_MARKET_GAIN",
        "Current redemption market gain before the prior-inclusion exclusion.",
    ),
    (
        "EXPLAIN_FARM_N_CCC_PAID_DEFICIENCY",
        "Required deficiency actually paid this year for an income-included loan: included once in farm expenses, not a reduction of gross receipts or a repeat deduction of previously taxed principal.",
    ),
    (
        "EXPLAIN_FARM_N_CCC_RETAINED_BASIS",
        "Basis of unsold commodities; retain individual loan and redemption records for later disposition.",
    ),
    (
        "EXPLAIN_FARM_N_CCC_SALES_NET",
        "Sale proceeds less commodity basis; losses remain deductible farm losses.",
    ),
    (
        "EXPLAIN_FARM_N_CCC_SALES_NET_CASH",
        "Recognized CCC commodity sales before the farm's shared opening/closing inventory adjustment; do not deduct cash-method basis again.",
    ),
    (
        "EXPLAIN_FARM_N_CCC_SURPLUS",
        "Additional CCC proceeds recognized in 2026 under the farm accounting method, including after an earlier forfeiture; principal and released deficiencies are not deducted again.",
    ),
    (
        "EXPLAIN_FARM_N_CCC_TAXABLE_MARKET_GAIN",
        "Market gain is taxable only for loans not previously included in income.",
    ),
    (
        "EXPLAIN_FARM_N_CCC_WITHHOLDING",
        "Actual current-year federal withholding, independent of principal inclusion.",
    ),
    (
        "EXPLAIN_FARM_N_CLOSING_INVENTORY",
        "Closing tax inventory, added once in Schedule F Part III.",
    ),
    (
        "EXPLAIN_FARM_N_CONSERVATION_INCOME_LIMIT",
        "Shared section 175 limit: 25% of the same taxpayer's combined supported gross farming income, before farm operating expenses.",
    ),
    (
        "EXPLAIN_FARM_N_COOPERATIVE_DISTRIBUTIONS",
        "Total Form 1099-PATR distributions routed to this farm: patronage dividends, nonpatronage distributions, per-unit retain allocations and redeemed nonqualified notices (Schedule F line 3a).",
    ),
    (
        "EXPLAIN_FARM_N_COOPERATIVE_TAXABLE",
        "Taxable cooperative distributions after the personal-purchase dividends reported on line 3a only (Schedule F line 3b).",
    ),
    (
        "EXPLAIN_FARM_N_CRP_N_INCOME",
        "Taxable CRP income included once in farm profit, on Schedule F lines 4a and 4b (39a and 39b on the accrual method); an SE exclusion does not remove this income.",
    ),
    (
        "EXPLAIN_FARM_N_CRP_N_SE_EXCLUSION",
        "Schedule SE line 1b: qualifying payments excluded from SE and applicable earned-income limits, not taxable farm profit.",
    ),
    ("EXPLAIN_FARM_N_CRP_N_WITHHOLDING", "Actual current-year federal withholding credited once."),
    (
        "EXPLAIN_FARM_N_DEPRECIATION",
        "Depreciation and section 179 expense: bonus, MACRS and the allowed section 179 deduction.",
    ),
    (
        "EXPLAIN_FARM_N_EXPENSES",
        "Farm operating expenses and supported shared deductions, including their applicable limits.",
    ),
    ("EXPLAIN_FARM_N_EXPENSE_LINE11_CHEMICALS", "Schedule F line 11: chemicals."),
    (
        "EXPLAIN_FARM_N_EXPENSE_LINE13_CUSTOM_HIRE",
        "Schedule F line 13: custom hire (machine work) where the operator furnished the equipment; equipment you rented and operated yourself is line 24a.",
    ),
    (
        "EXPLAIN_FARM_N_EXPENSE_LINE15_EMPLOYEE_BENEFIT_PROGRAMS",
        "Schedule F line 15: employee benefit programs other than pension and profit-sharing plans on line 23.",
    ),
    ("EXPLAIN_FARM_N_EXPENSE_LINE16_FEED", "Schedule F line 16: feed."),
    (
        "EXPLAIN_FARM_N_EXPENSE_LINE17_FERTILIZERS_AND_LIME",
        "Schedule F line 17: fertilizers and lime.",
    ),
    (
        "EXPLAIN_FARM_N_EXPENSE_LINE18_FREIGHT_AND_TRUCKING",
        "Schedule F line 18: freight and trucking; transportation of livestock bought for resale is part of its cost instead.",
    ),
    (
        "EXPLAIN_FARM_N_EXPENSE_LINE19_GASOLINE_FUEL_AND_OIL",
        "Schedule F line 19: gasoline, fuel and oil.",
    ),
    (
        "EXPLAIN_FARM_N_EXPENSE_LINE20_INSURANCE",
        "Schedule F line 20: insurance, other than health.",
    ),
    (
        "EXPLAIN_FARM_N_EXPENSE_LINE21A_MORTGAGE_INTEREST",
        "Schedule F line 21a: mortgage interest paid to banks or other financial institutions.",
    ),
    (
        "EXPLAIN_FARM_N_EXPENSE_LINE21C_OTHER_INTEREST",
        "Schedule F line 21c: other farm interest; interest on a loan to buy a farm vehicle belongs on line 21b.",
    ),
    (
        "EXPLAIN_FARM_N_EXPENSE_LINE22_LABOR_HIRED",
        "Schedule F line 22: labor hired for farm work, wages and contract labor, less any employment credits.",
    ),
    (
        "EXPLAIN_FARM_N_EXPENSE_LINE23_PENSION_AND_PROFIT_SHARING",
        "Schedule F line 23: contributions to employees' pension, profit-sharing or annuity plans; your own contributions as a self-employed person are Schedule 1 line 16.",
    ),
    (
        "EXPLAIN_FARM_N_EXPENSE_LINE24A_RENT_VEHICLES_EQUIPMENT",
        "Schedule F line 24a: rent or lease of vehicles, machinery and equipment.",
    ),
    (
        "EXPLAIN_FARM_N_EXPENSE_LINE24B_RENT_OTHER",
        "Schedule F line 24b: rent or lease of other property, such as pasture or farmland.",
    ),
    (
        "EXPLAIN_FARM_N_EXPENSE_LINE25_REPAIRS",
        "Schedule F line 25: repairs and maintenance of farm buildings, machinery and equipment.",
    ),
    ("EXPLAIN_FARM_N_EXPENSE_LINE26_SEEDS_AND_PLANTS", "Schedule F line 26: seeds and plants."),
    (
        "EXPLAIN_FARM_N_EXPENSE_LINE27_STORAGE_AND_WAREHOUSING",
        "Schedule F line 27: storage and warehousing.",
    ),
    ("EXPLAIN_FARM_N_EXPENSE_LINE28_SUPPLIES", "Schedule F line 28: supplies."),
    (
        "EXPLAIN_FARM_N_EXPENSE_LINE29_TAXES",
        "Schedule F line 29: taxes on farm business assets and the employer's payroll taxes.",
    ),
    (
        "EXPLAIN_FARM_N_EXPENSE_LINE30_UTILITIES",
        "Schedule F line 30: utilities for business use on the farm.",
    ),
    (
        "EXPLAIN_FARM_N_EXPENSE_LINE31_VETERINARY_BREEDING_AND_MEDICINE",
        "Schedule F line 31: veterinary, breeding and medicine.",
    ),
    (
        "EXPLAIN_FARM_N_EXPENSE_LINE32_ADVERTISING",
        "Schedule F lines 32a–32e: other expenses, advertising.",
    ),
    (
        "EXPLAIN_FARM_N_EXPENSE_LINE32_COMMISSIONS_AND_FEES",
        "Schedule F lines 32a–32e: other expenses, commissions and fees.",
    ),
    (
        "EXPLAIN_FARM_N_EXPENSE_LINE32_LEGAL_AND_PROFESSIONAL",
        "Schedule F lines 32a–32e: other expenses, legal and professional fees.",
    ),
    (
        "EXPLAIN_FARM_N_EXPENSE_LINE32_MEALS",
        "Schedule F lines 32a–32e: other expenses, deductible meals after the 50% ordinary or 80% DOT hours-of-service limit.",
    ),
    (
        "EXPLAIN_FARM_N_EXPENSE_LINE32_OFFICE",
        "Schedule F lines 32a–32e: other expenses, office expense.",
    ),
    (
        "EXPLAIN_FARM_N_EXPENSE_LINE32_STARTUP_AMORTIZATION",
        "Schedule F lines 32a–32f: the start-up cost deduction, the amortization of start-up costs and the section 197 amortization.",
    ),
    (
        "EXPLAIN_FARM_N_EXPENSE_LINE32_PREPAID_SUPPLIES",
        "Schedule F lines 32a–32f: the deduction for prepaid farm supplies and released earlier prepayments, after the one-half-of-other-expenses ceiling.",
    ),
    (
        "EXPLAIN_FARM_N_EXPENSE_LINE32_CCC_DEFICIENCY",
        "Schedule F lines 32a–32f: a required CCC loan deficiency payment actually paid in 2026.",
    ),
    (
        "EXPLAIN_FARM_N_EXPENSE_LINE32_OTHER",
        "Schedule F lines 32a–32e: other expenses entered with their own description.",
    ),
    ("EXPLAIN_FARM_N_EXPENSE_LINE32_TRAVEL", "Schedule F lines 32a–32e: other expenses, travel."),
    (
        "EXPLAIN_FARM_N_GROSS_INCOME",
        "Farm gross income after product basis or the accrual inventory reconciliation, before operating expenses, including fuel-tax credit or refund income and the 2026 part of a section 481(a) increase (line 8), any Form 1099-G box 7 agriculture-program payment and taxable Form 1099-PATR distributions routed to this farm.",
    ),
    (
        "EXPLAIN_FARM_N_LAND_N_N_CONSERVATION_RECAPTURE",
        "Section 1252 ordinary recapture, limited by gain and the holding-period percentage of prior allowed conservation deductions.",
    ),
    (
        "EXPLAIN_FARM_N_LAND_N_N_GAIN",
        "Land proceeds less selling expenses and retained basis (for an installment sale, the part in 2026's payments, Form 6252 line 24); not Schedule F operating receipts or self-employment earnings.",
    ),
    (
        "EXPLAIN_FARM_N_LAND_N_N_ORDINARY",
        "Direct Form 4797 ordinary amount before downstream return coordination; includes all gain/loss for land held one year or less.",
    ),
    (
        "EXPLAIN_FARM_N_LAND_N_N_SECTION1231",
        "Eligible long-held land amount after conservation recapture, before return-wide section 1231 netting and five-year lookback.",
    ),
    (
        "EXPLAIN_FARM_N_NET_PROFIT",
        "Schedule F profit included once in Schedule 1, self-employment earnings and applicable QBI/deduction calculations, including any Form 1099-G box 7 agriculture-program payment and taxable Form 1099-PATR distributions routed to this farm.",
    ),
    (
        "EXPLAIN_FARM_N_N_CONSERVATION_ASSESSMENT_AVAILABLE",
        "District assessment amounts available this year, including statutory acceleration on the farm owner's final return; subject next to the 25% farming-income limit.",
    ),
    (
        "EXPLAIN_FARM_N_N_CONSERVATION_ASSESSMENT_DEFERRED",
        "Paid assessment costs treated as paid in later years of the nine-year spread; separate from the 25%-limited carryforward and future unpaid installments.",
    ),
    (
        "EXPLAIN_FARM_N_N_CONSERVATION_BASIS_ADDITION",
        "Capitalized conservation payments to retain with land basis; no current deduction or automatic update of another property record.",
    ),
    (
        "EXPLAIN_FARM_N_N_CONSERVATION_CARRYFORWARD",
        "Unused section 175 deduction for later years, subject again to their gross-farm-income limit; do not also add this balance to land basis.",
    ),
    (
        "EXPLAIN_FARM_N_N_CONSERVATION_DEDUCTION",
        "Section 175 costs and prior unused deduction, limited to 25% of gross farming income; included once in Schedule F expenses.",
    ),
    (
        "EXPLAIN_FARM_N_N_CONSERVATION_EXPIRED",
        "The deceased farm owner's unused section 175 income-limited balance expires; it is not transferred to a survivor or estate and is not added to land basis.",
    ),
    (
        "EXPLAIN_FARM_N_N_CROP_INSURANCE_BEFORE_DEFERRAL",
        "Current crop-insurance and disaster proceeds recognized before the election; exclude duplicate subsidy entries.",
    ),
    (
        "EXPLAIN_FARM_N_N_CROP_INSURANCE_CURRENT_TAXABLE",
        "Current-year proceeds less the amount elected into 2027 (Schedule F line 6b).",
    ),
    (
        "EXPLAIN_FARM_N_N_CROP_INSURANCE_DEFERRED_N",
        "Eligible current physical-loss proceeds elected into 2027; retain the amount for next year.",
    ),
    (
        "EXPLAIN_FARM_N_N_CROP_INSURANCE_PRIOR_INCLUDED",
        "2025 deferred proceeds included once in 2026; not eligible for another deferral.",
    ),
    (
        "EXPLAIN_FARM_N_N_CROP_INSURANCE_TAXABLE",
        "Insurance and disaster income included once in farm gross income, SE earnings and QBI.",
    ),
    (
        "EXPLAIN_FARM_N_N_CROP_INSURANCE_WITHHOLDING",
        "Actual 2026 federal withholding remains a payment credit even if eligible income is deferred.",
    ),
    (
        "EXPLAIN_FARM_N_OPENING_INVENTORY",
        "Opening tax inventory, subtracted once in Schedule F Part III.",
    ),
    (
        "EXPLAIN_FARM_N_OTHER_ACCRUAL_INCOME",
        "Recognized custom-hire, ordinary program and other operating income.",
    ),
    (
        "EXPLAIN_FARM_N_PRODUCT_PURCHASES",
        "Accrued product purchases, subtracted once; unsold inventory remains in closing valuation.",
    ),
    (
        "EXPLAIN_FARM_N_PROGRAM_PAYMENTS",
        "Agricultural program payments received: program payments, Conservation Reserve Program payments, CCC loan market gain, any Form 1099-G box 7 payment routed to this farm and any cost-sharing payment (Schedule F line 4a).",
    ),
    (
        "EXPLAIN_FARM_N_PROGRAM_TAXABLE",
        "Taxable agricultural program payments: the fully taxable payments, CRP payments and the market gain on a loan not included in income (Schedule F line 4b).",
    ),
    (
        "EXPLAIN_FARM_N_RAISED_SALES",
        "Schedule F line 2: raised products and market livestock receipts, including a redeemed CCC commodity sold whose loan was not included in income.",
    ),
    (
        "EXPLAIN_FARM_N_RESALE_BASIS",
        "Schedule F line 1b: paid basis of purchased products sold, deducted once; a redeemed CCC commodity's basis is its redemption payment.",
    ),
    (
        "EXPLAIN_FARM_N_RESALE_PROCEEDS",
        "Schedule F line 1a: purchased products sold, proceeds before basis, including a redeemed CCC commodity sold whose loan was included in income.",
    ),
    (
        "EXPLAIN_FARM_N_SUPPLIES_N_DEDUCTION",
        "Current deduction for this farm's prepayments and released costs, included once in operating expenses. Supported same-owner cash farms share one expense ceiling; the deduction retains its source farm.",
    ),
    (
        "EXPLAIN_FARM_N_SUPPLIES_N_ELIGIBLE",
        "Eligible new supply prepayments and payment-method unsold resale-poultry purchases, before the expense ceiling. Supply purchase, business-purpose and income-distortion tests apply separately.",
    ),
    (
        "EXPLAIN_FARM_N_SUPPLIES_N_POULTRY_DISPOSED_COST",
        "Current purchase costs and previously undeducted costs of resale poultry disposed of this year. Included in the total supply deduction below; do not deduct again against sales receipts.",
    ),
    (
        "EXPLAIN_FARM_N_SUPPLIES_N_PRODUCTION_POULTRY_RELEASED",
        "Production-poultry costs paid in 2025 but deferred by its prepaid ceiling release in 2026. Included once in the released-cost subtotal and current deduction; sale is not a prerequisite.",
    ),
    (
        "EXPLAIN_FARM_N_SUPPLIES_N_RELEASED",
        "Earlier undeducted supplies consumed, resale-poultry costs disposed of, and eligible 2025 production-poultry deferrals released this year; deducted once and included in other deductible expenses.",
    ),
    (
        "EXPLAIN_FARM_N_SUPPLIES_N_REMAINING",
        "Undeducted supply cost retained after this year's deduction; preserve identified cost and consumption records for subsequent returns.",
    ),
    (
        "EXPLAIN_FEDERAL_RULES",
        "Annual taxable ABLE earnings after qualified disability expenses enter Schedule 1 line 8q. Form 5329 Part II adds 10% per recipient; principal and qualifying rollovers or post-death payments are excluded. Timely excess-contribution corrections include earnings in the contribution year without the 10% tax.",
    ),
    (
        "EXPLAIN_FEDERAL_RULES_10",
        "Unused capital loss after the annual limit and actual use against unfloored taxable income with the section 151 enhanced senior deduction added back; short-term losses are used first. The additional age/blindness standard deduction is not added back. A selected section 1341 credit uses the refigured restoration-year balance under section 1341(b)(4).",
    ),
    (
        "EXPLAIN_FEDERAL_RULES_11",
        "Preferential income included in Form 8615 line 5 after the applicable deduction allocation. Available for other affected children's family worksheets.",
    ),
    (
        "EXPLAIN_FEDERAL_RULES_12",
        "Original gain source before Form 8615 deduction allocation, for coordinating another child's return. Special-rate amounts precede the net-capital-gain cap.",
    ),
    (
        "EXPLAIN_FEDERAL_RULES_13",
        "Separate AMT capital netting, personal-loss disallowance and special-rate pools using retained sale basis and AMT loss history; does not change regular AGI or QBI.",
    ),
    (
        "EXPLAIN_FEDERAL_RULES_14",
        "Separate AMT capital-loss carryover after actual use against unfloored AMTI, before the AMT exemption. Preserve short/long character and retain separately from regular carryovers. A selected section 1341 credit uses the refigured restoration-year balance.",
    ),
    (
        "EXPLAIN_FEDERAL_RULES_15",
        "Aggregate investment-related itemized deductions are limited to total itemized deductions allowed after section 68. The 2026 law applies that limitation after Form 4952 and has no former investment-interest exception.",
    ),
    (
        "EXPLAIN_FEDERAL_RULES_16",
        "Current-year loss carryover after separate regular/AMT debt reduction. The original loss and current tax are unchanged; carryovers retain only the post-reduction balance.",
    ),
    (
        "EXPLAIN_FEDERAL_RULES_2",
        "Combined Form 8853 Section B. Taxable distributions enter Schedule 1 line 8e; the separate additional tax enters Schedule 2. Medical withdrawals are not also deductible.",
    ),
    (
        "EXPLAIN_FEDERAL_RULES_3",
        "2026 regular IRA contribution treatment. Nondeductible traditional amounts require Form 8606 and basis records; this prototype prepares no forms.",
    ),
    (
        "EXPLAIN_FEDERAL_RULES_5",
        "Form 4952 investment-property pool, distinct from Schedule D totals. Personal/home dispositions are excluded; confirmed investment losses and carryovers are netted, including both ordinary and capital portions of section 1244 losses. Nonbusiness royalties contribute after directly deductible expenses. Qualified dividends and net capital gain are excluded from ordinary investment income without an election.",
    ),
    (
        "EXPLAIN_FEDERAL_RULES_6",
        "2026 Schedule 1-A deduction after claimant eligibility, annual limits and income phaseout. Does not reduce AGI or payroll tax.",
    ),
    (
        "EXPLAIN_FEDERAL_RULES_7",
        "Qualified/nonqualified portion of the same excess-business-loss NOL carryforward, allocated from loss-only source activities. Qualified suspended loss is excluded from current QBI carryforward until deductible; these details are not additional NOL amounts.",
    ),
    (
        "EXPLAIN_FEDERAL_RULES_8",
        "Separate regular and AMT investment-interest limits. Disallowed expense carries forward; choosing the standard deduction does not carry forward interest that the income limit allowed.",
    ),
    (
        "EXPLAIN_FEDERAL_RULES_9",
        "Form 8615 line 1, the child's unearned income. Form 8615 applies when it is more than $2,700.",
    ),
    (
        "EXPLAIN_FINAL_BOND_INTEREST_N",
        "Mandatory annual-method accrual or elected deferred-method interest through death: redemption value less cost and previously reported interest. Preserve this record for the recipient; do not include it again in another income entry.",
    ),
    (
        "EXPLAIN_FINAL_BOND_RECIPIENT_INTEREST_N",
        "Unreported pre-death interest left for the estate or beneficiary under the no-election choice. Post-death earnings are separate and are not calculated here. This amount is not included in the decedent's taxable income.",
    ),
    (
        "EXPLAIN_FIRST_HOME_USED_TAXPAYER",
        "Lifetime first-home IRA amount used through 2026, including the opening history. Retain for later returns.",
    ),
    (
        "EXPLAIN_FOREIGN_AMT_HIGH_TAX_NET_INCOME",
        "Net passive income reclassified independently for AMT after AMT-allowed deductions and the applicable 26% or 28% rate.",
    ),
    (
        "EXPLAIN_FOREIGN_AMT_HIGH_TAX_TAXES",
        "Foreign taxes moved with high-taxed income to the AMT general category; regular classification can differ.",
    ),
    (
        "EXPLAIN_FOREIGN_AMT_LINE_N_RATE_ADJUSTMENT",
        "AMT Worksheet for Line 18: Form 6251 line 23 plus 0.4643 of line 30, 0.2857 of line 33 and 0.1071 of line 36; zero under the adjustment exception or when no adjustment is required.",
    ),
    (
        "EXPLAIN_FOREIGN_AMT_QUALIFIED_DIVIDEND_ADJUSTMENT",
        "AMT Form 1116 line 1a: foreign qualified dividends taxed at 0% are left out and those taxed at 15% or 20% are kept at 0.5357 or 0.7143.",
    ),
    (
        "EXPLAIN_FOREIGN_BUSINESS_N_N",
        "One Form 2555 limit for this person's combined qualifying business receipts and employee pay. Business exclusion is its proportional share. Excluded receipts disallow proportional business expenses and half-SE deduction. Net exclusion reduces Schedule 1 income while SE tax remains computed on full business profit.",
    ),
    (
        "EXPLAIN_FOREIGN_EARNED_EXCLUSION",
        "Gross foreign earned-income exclusion before restoring deductions related to excluded business receipts; limited to $132,900 per person in 2026, prorated for qualifying days.",
    ),
    (
        "EXPLAIN_FOREIGN_EARNED_N_N_INCOME",
        "Form 2555 line 26: this person's foreign earned income, W-2 pay for work abroad plus pay from a foreign employer plus receipts of businesses run abroad.",
    ),
    (
        "EXPLAIN_FOREIGN_EARNED_N_N_DAYS",
        "Form 2555 lines 31 and 38: the qualifying days in 2026 of the bona fide residence or the 12-month physical presence period, which prorate the exclusion limit.",
    ),
    (
        "EXPLAIN_FOREIGN_GENERAL_EXCLUDED_TAX_REDUCTION",
        "Form 1116 line 12: foreign tax on foreign earned income excluded on Form 2555 is not creditable (section 911(d)(6)); the employment taxes fall by the excluded share.",
    ),
    (
        "EXPLAIN_FOREIGN_GENERAL_AMT_EXCLUDED_TAX_REDUCTION",
        "The same reduction of foreign taxes on excluded foreign earned income applies to the alternative minimum tax foreign tax credit.",
    ),
    (
        "EXPLAIN_FOREIGN_EMPLOYEE_N_GROSS_EXCLUSION",
        "Employee share of this person's combined foreign earned-income exclusion. Business receipts share the same cap; this amount is included in the total gross exclusion, not added again.",
    ),
    (
        "EXPLAIN_FOREIGN_GENERAL_AMT_UNUSED_TAX",
        "Separate AMT foreign tax balance; keep it distinct from the regular-tax balance.",
    ),
    (
        "EXPLAIN_FOREIGN_GENERAL_UNUSED_TAX",
        "Current-year foreign tax exceeding the category limitation; review the one-year carryback and ten-year carryforward rules separately.",
    ),
    (
        "EXPLAIN_FOREIGN_HIGH_TAX_NET_INCOME",
        "Net passive income reclassified to general income after regular-tax expense allocation and the 37% high-tax test.",
    ),
    (
        "EXPLAIN_FOREIGN_HIGH_TAX_TAXES",
        "Foreign taxes moved with high-taxed income to the regular general category.",
    ),
    (
        "EXPLAIN_FOREIGN_HOUSING_DEDUCTION",
        "Self-employed housing deduction, including allowed prior-year carryover. Reported separately from the income exclusion in Schedule 1 adjustments.",
    ),
    (
        "EXPLAIN_FOREIGN_HOUSING_EXCLUSION",
        "Housing exclusion allocated to employer-provided foreign income before the earned-income exclusion.",
    ),
    (
        "EXPLAIN_FOREIGN_HOUSING_N_N",
        "Current housing deduction is limited by foreign earned income after gross exclusions and used before the prior-year carryover. Unused current amount carries one year; unused prior-year amount expires.",
    ),
    (
        "EXPLAIN_FOREIGN_HOUSING_N_N_BASE",
        "This claimant's full qualifying-period base housing amount; each separate claimant subtracts their own base.",
    ),
    (
        "EXPLAIN_FOREIGN_HOUSING_N_N_EXCLUSION",
        "Eligible expenses limited by location, less the base housing amount and nontaxable housing allowances, capped at this claimant's employer-provided foreign earnings.",
    ),
    (
        "EXPLAIN_FOREIGN_HOUSING_N_N_EXPENSES",
        "This claimant's eligible housing expenses before the location limit and base amount; excludes amounts allocated to the other spouse.",
    ),
    (
        "EXPLAIN_FOREIGN_HOUSING_N_N_LIMIT",
        "Sum of published location expense limits for this claimant's qualifying stays, combining repeated locations.",
    ),
    (
        "EXPLAIN_FOREIGN_HOUSING_N_N_NONTAXABLE_ALLOWANCES",
        "Tax-free housing allowance paid to the claimant or spouse for this qualifying period; reduces the limited housing amount, not foreign wages.",
    ),
    (
        "EXPLAIN_FOREIGN_INVESTMENT_N_GROUP_N",
        "Allowed investment interest allocated by current-year average adjusted basis, including released prior disallowed interest. Regular deduction reflects section 68; AMT uses its independent deduction and basis. Groups0–4 precede high-tax reclassification; group5 is domestic.",
    ),
    (
        "EXPLAIN_FOREIGN_K3_DEFINITELY_RELATED_DEDUCTIONS",
        "Form 1116 line 2: Schedule K-3 Part II section 2 deductions of columns (c) and (d) (lines 25, 26, 31 through 37 and 44 through 50), already allocated to the passive and general categories; column (f) deductions are apportioned by gross income on line 3b.",
    ),
    (
        "EXPLAIN_FOREIGN_K3_GROSS_INCOME_CORRECTION",
        "Form 1116 line 3e: each Schedule K-3's Part II line 24 gross income in place of its K-1's net Schedule E amount and its interest and dividends (box 5 and 6a; Form 1120-S boxes 4 and 5a) already in adjusted gross income.",
    ),
    (
        "EXPLAIN_FOREIGN_K3_INTEREST_EXPENSE",
        "Form 1116 line 4b: Schedule K-3 interest of lines 39 through 43 in columns (c) and (d), and the column (f) business and passive activity interest apportioned by the Part III Section 2 line 6b and 6d assets of your Schedule K-3s.",
    ),
    (
        "EXPLAIN_FOREIGN_K3_TAX_REDUCTION",
        "Form 1116 line 12: the Schedule K-3 Part III reduction of taxes (line 2), spread over the category's rows by income, that reduced the taxes of line 8.",
    ),
    (
        "EXPLAIN_FOREIGN_LINE_N_RATE_ADJUSTMENT",
        "Worksheet for Line 18: 0% income plus 0.5946 of 15% income, 0.4595 of 20% income, 0.3243 of 25% gain and 0.2432 of 28% gain from the Qualified Dividends and Capital Gain Tax Worksheet or Schedule D Tax Worksheet; zero under the adjustment exception or when no adjustment is required.",
    ),
    (
        "EXPLAIN_FOREIGN_N_DOMESTIC_DEDUCTIONS",
        "Eligible U.S. charitable deductions, domestic state-income-tax and investment-interest shares, and elected small-foreign-income mortgage interest; excluded from gross-ratio foreign expense allocation.",
    ),
    (
        "EXPLAIN_FOREIGN_N_N_N",
        "2026 excess absorbed by hypothetical 2025 credit capacity after older taxes. With the deduction choice unchanged, this amount gives no credit, deduction or refund and cannot carry forward.",
    ),
    (
        "EXPLAIN_FOREIGN_N_N_N_2",
        "2026 excess allocated to remaining 2025 category capacity after older taxes. This is not a refund; the prior return and dependent credits require recomputation.",
    ),
    (
        "EXPLAIN_FOREIGN_N_N_N_CARRY_EXPIRED",
        "Unused 2016 foreign taxes expire after their tenth carryforward year, 2026.",
    ),
    (
        "EXPLAIN_FOREIGN_N_N_N_CARRY_USED",
        "Total prior-year foreign tax used against this year's remaining category limit.",
    ),
    (
        "EXPLAIN_FOREIGN_N_N_N_NEW_CARRYFORWARD",
        "Unused 2026 taxes remaining after the one-year carryback or deduction-year absorption review, eligible for up to ten subsequent years. Separate from retained older balances.",
    ),
    (
        "EXPLAIN_FOREIGN_N_N_N_NEW_OVERALL_DOMESTIC_LOSS",
        "New overall domestic loss account for this category: domestic loss offset this category's positive foreign income. Retain for future domestic-to-foreign recharacterization.",
    ),
    (
        "EXPLAIN_FOREIGN_N_N_N_NEW_OVERALL_FOREIGN_LOSS",
        "New overall foreign loss account: this category's remaining loss offset domestic taxable income after other foreign categories were exhausted. Retain for future recapture.",
    ),
    (
        "EXPLAIN_FOREIGN_N_N_N_NEW_SEPARATE_LOSS",
        "Current separate limitation loss offset: this category reduced the other category. Retained opposing accounts are netted separately before recapture; keep the final remaining balance and direction.",
    ),
    (
        "EXPLAIN_FOREIGN_N_N_N_OVERALL_DOMESTIC_LOSS_RECAPTURED",
        "Domestic income recharacterized into this foreign category under the statutory50% rule; excludes U.S. income created by overall foreign loss recapture.",
    ),
    (
        "EXPLAIN_FOREIGN_N_N_N_OVERALL_DOMESTIC_LOSS_REMAINING",
        "Closing overall domestic loss account after current additions and recapture; retain for later returns.",
    ),
    (
        "EXPLAIN_FOREIGN_N_N_N_OVERALL_FOREIGN_LOSS_RECAPTURED",
        "Foreign income recharacterized as domestic under the overall foreign loss account; includes an eligible additional election.",
    ),
    (
        "EXPLAIN_FOREIGN_N_N_N_OVERALL_FOREIGN_LOSS_REMAINING",
        "Closing overall foreign loss account after current additions and recapture; retain for later returns.",
    ),
    (
        "EXPLAIN_FOREIGN_N_N_N_RETAINED_CARRYFORWARD",
        "Remaining 2017–2025 balances for later years. Newly generated 2026 unused taxes are separate until their prior-year carryback is resolved.",
    ),
    (
        "EXPLAIN_FOREIGN_N_N_N_SEPARATE_LOSS_NETTED",
        "Opposing current/prior separate limitation loss accounts canceled before recapture.",
    ),
    (
        "EXPLAIN_FOREIGN_N_N_N_SEPARATE_LOSS_RECAPTURED",
        "Remaining income recharacterized from this category to the other foreign category after overall foreign loss recapture.",
    ),
    (
        "EXPLAIN_FOREIGN_N_N_N_SEPARATE_LOSS_REMAINING",
        "Closing separate limitation loss account after current additions, opposing-account netting and recapture; retain its direction.",
    ),
    (
        "EXPLAIN_FOREIGN_N_PASSIVE_LOSS_REALLOCATED",
        "Excess deductions of passive withholding groups proportionately reduce positive net income in the other passive groups, without reducing any below zero, before high-tax classification. This step does not offset employment or domestic income.",
    ),
    (
        "EXPLAIN_FOREIGN_QUALIFIED_DIVIDEND_ADJUSTMENT",
        "Form 1116 line 1a: foreign qualified dividends taxed at 0% are left out and those taxed at 15% or 20% are kept at 0.4054 or 0.5405, split ratably across the rate bands.",
    ),
    (
        "EXPLAIN_FOREIGN_STATE_INCOME_TAX_GROUP_N",
        "Allowed state income-tax deduction after SALT and section 68 limits, allocated using the income underlying each payment. Groups0–4 retain the existing foreign withholding classification; group5 is domestic. AMT adds back these taxes.",
    ),
    (
        "EXPLAIN_FOREIGN_TAX_CREDIT",
        "Foreign tax credit after the selected election or separate income-category limits.",
    ),
    (
        "EXPLAIN_FOREIGN_WAGES",
        "Foreign-source employee pay not already entered on a W-2; translated to U.S. dollars from payment records.",
    ),
    (
        "EXPLAIN_FORM_1040_ES_LINE_1",
        "Adjusted gross income expected in 2027: this return's Form 1040 line 11.",
    ),
    (
        "EXPLAIN_FORM_1040_ES_LINE_10",
        "Other taxes: this return's Schedule 2 Part II taxes the instructions list, including household employment tax under Exception 1 (income tax will be withheld, or estimated payments are required without it).",
    ),
    (
        "EXPLAIN_FORM_1040_ES_LINE_10_2",
        "Other taxes: this return's Schedule 2 Part II taxes the instructions list. Household employment tax is left out: no income tax will be withheld and no estimated payment is required without it (Exception 1).",
    ),
    (
        "EXPLAIN_FORM_1040_ES_LINE_10_3",
        "Other taxes: this return's Schedule 2 Part II taxes the instructions list (Additional Medicare, net investment income, retirement and account additional taxes, education-credit recapture), without Forms 4137/8919, uncollected tip taxes and Form 8828 recapture.",
    ),
    ("EXPLAIN_FORM_1040_ES_LINE_11A", "Lines 8, 9 and 10 added."),
    (
        "EXPLAIN_FORM_1040_ES_LINE_11B",
        "Refundable credits: this return's earned income credit, additional child tax credit, fuel tax credit, net premium tax credit, refundable American opportunity and adoption credits and section 1341 credit.",
    ),
    (
        "EXPLAIN_FORM_1040_ES_LINE_11C",
        "Total 2027 estimated tax: line 11a less line 11b, not below zero.",
    ),
    (
        "EXPLAIN_FORM_1040_ES_LINE_12A",
        "66⅔% of line 11c: at least two-thirds of your gross income is from farming or fishing.",
    ),
    ("EXPLAIN_FORM_1040_ES_LINE_12A_2", "90% of line 11c."),
    (
        "EXPLAIN_FORM_1040_ES_LINE_12B",
        "110% of this return's 2026 tax ({0,number,currency}: Form 1040 line 24 without Forms 4137/8919, uncollected tip taxes, Form 8828 recapture and excess-contribution excises, less line 11b's credits), because this return's AGI is above {1,number,currency}.",
    ),
    (
        "EXPLAIN_FORM_1040_ES_LINE_12B_2",
        "100% of this return's 2026 tax ({0,number,currency}); the 110% rule does not apply to farmers and fishermen.",
    ),
    (
        "EXPLAIN_FORM_1040_ES_LINE_12B_3",
        "100% of this return's 2026 tax ({0,number,currency}: Form 1040 line 24 without Forms 4137/8919, uncollected tip taxes, Form 8828 recapture and excess-contribution excises, less line 11b's credits). This return covers all of 2026.",
    ),
    (
        "EXPLAIN_FORM_1040_ES_LINE_12C",
        "Required annual payment to avoid a penalty: the smaller of lines 12a and 12b.",
    ),
    (
        "EXPLAIN_FORM_1040_ES_LINE_13",
        "Income tax expected to be withheld during 2027: this return's Form 1040 line 25d.",
    ),
    (
        "EXPLAIN_FORM_1040_ES_LINE_13_2",
        "Income tax expected to be withheld during 2027, as entered on the payments page.",
    ),
    (
        "EXPLAIN_FORM_1040_ES_LINE_14A",
        "Line 12c less line 13. Zero or less: no estimated tax payments are required.",
    ),
    (
        "EXPLAIN_FORM_1040_ES_LINE_14B",
        "Line 11c less line 13. Under $1,000: no estimated tax payments are required.",
    ),
    (
        "EXPLAIN_FORM_1040_ES_LINE_15",
        "The whole of line 14a: farmers and fishermen make one payment, due January 18, 2028.",
    ),
    (
        "EXPLAIN_FORM_1040_ES_LINE_15_2",
        "One quarter of line 14a ({0,number,currency} exactly), due April 15, June 15 and September 15, 2027 and January 18, 2028, less any 2026 overpayment applied to the first installment.",
    ),
    (
        "EXPLAIN_FORM_1040_ES_LINE_1_2",
        "Adjusted gross income expected in 2027, as entered on the payments page.",
    ),
    (
        "EXPLAIN_FORM_1040_ES_LINE_2A",
        "Deductions: this return's itemized deductions, or its standard deduction plus the nonitemizer cash gifts and qualified disaster loss (Form 1040 line 12e), at the 2026 amounts.",
    ),
    (
        "EXPLAIN_FORM_1040_ES_LINE_2A_2",
        "Deductions expected in 2027, as entered on the payments page.",
    ),
    (
        "EXPLAIN_FORM_1040_ES_LINE_2B",
        "Qualified business income deduction: this return's Form 1040 line 13b.",
    ),
    (
        "EXPLAIN_FORM_1040_ES_LINE_2C",
        "Schedule 1-A additional deductions: this return's Form 1040 line 13a.",
    ),
    ("EXPLAIN_FORM_1040_ES_LINE_2D", "Lines 2a, 2b and 2c added."),
    ("EXPLAIN_FORM_1040_ES_LINE_3", "Line 1 less line 2d, not below zero."),
    (
        "EXPLAIN_FORM_1040_ES_LINE_4",
        "Tax on line 3 by the Qualified Dividends and Capital Gain Tax Worksheet (Pub. 505) at the {0,number,0} Tax Rate Schedules, with this return's {1,number,currency} of qualified dividends and net capital gain.",
    ),
    (
        "EXPLAIN_FORM_1040_ES_LINE_4_2",
        "Tax on line 3 by the {0,number,0} Tax Rate Schedules (the {1,number,0} schedules are not published).",
    ),
    ("EXPLAIN_FORM_1040_ES_LINE_5", "Alternative minimum tax: this return's Form 6251."),
    (
        "EXPLAIN_FORM_1040_ES_LINE_6",
        "Lines 4 and 5, plus the {0,number,currency} positive Form 8978 adjustment included on this return's Form 1040 line 16.",
    ),
    (
        "EXPLAIN_FORM_1040_ES_LINE_6_2",
        "Lines 4 and 5 added; no other tax is included on this return's Form 1040 line 16.",
    ),
    (
        "EXPLAIN_FORM_1040_ES_LINE_7",
        "Nonrefundable credits: this return's Form 1040 line 21 without the clean vehicle, residential clean energy and refueling credits, which cannot be claimed again.",
    ),
    (
        "EXPLAIN_FORM_1040_ES_LINE_7_2",
        "Nonrefundable credits: this return's Form 1040 line 21. Withholding is not a credit here.",
    ),
    (
        "EXPLAIN_FORM_1040_ES_LINE_7_3",
        "Nonrefundable credits expected in 2027, as entered on the payments page.",
    ),
    ("EXPLAIN_FORM_1040_ES_LINE_8", "Line 6 less line 7, not below zero."),
    (
        "EXPLAIN_FORM_1040_ES_LINE_9",
        "Self-employment tax: this return's Schedule SE, at the 2026 $184,500 Social Security wage base.",
    ),
    ("EXPLAIN_FORM_2210_AI_LINE_N", "Schedule AI line 1: adjusted gross income for the period."),
    (
        "EXPLAIN_FORM_2210_AI_LINE_N_10",
        "Schedule AI line 25: the previous column's line 26 minus its line 27.",
    ),
    (
        "EXPLAIN_FORM_2210_AI_LINE_N_11",
        "Schedule AI line 27: the smaller of line 23 and line 26 (25% of Form 2210 line 9 plus line 25); this is Form 2210 line 10.",
    ),
    (
        "EXPLAIN_FORM_2210_AI_LINE_N_2",
        "Schedule AI line 3: annualized income, line 1 times 4, 2.4, 1.5 or 1.",
    ),
    (
        "EXPLAIN_FORM_2210_AI_LINE_N_3",
        "Schedule AI line 6: itemized deductions for the period, annualized the same way.",
    ),
    (
        "EXPLAIN_FORM_2210_AI_LINE_N_4",
        "Schedule AI line 11: line 3 minus the larger of line 6 and the standard deduction, the qualified business income deduction and the Schedule 1-A deductions.",
    ),
    ("EXPLAIN_FORM_2210_AI_LINE_N_5", "Schedule AI line 13: line 11, not less than zero."),
    (
        "EXPLAIN_FORM_2210_AI_LINE_N_6",
        "Schedule AI line 14: tax on line 13 from the Tax Table or Tax Computation Worksheet.",
    ),
    (
        "EXPLAIN_FORM_2210_AI_LINE_N_7",
        "Schedule AI line 15: annualized self-employment tax from Part II line 36, the prorated Social Security wage base applied.",
    ),
    (
        "EXPLAIN_FORM_2210_AI_LINE_N_8",
        "Schedule AI line 21: line 14 plus line 15 minus the year's credits, times the applicable percentage of 22.5%, 45%, 67.5% or 90%.",
    ),
    (
        "EXPLAIN_FORM_2210_AI_LINE_N_9",
        "Schedule AI line 23: line 21 minus the line 27 amounts of earlier columns, not less than zero.",
    ),
    (
        "EXPLAIN_FORM_2210_F_LINE_1",
        "Form 2210-F line 1: Form 1040 line 22, tax after nonrefundable credits.",
    ),
    (
        "EXPLAIN_FORM_2210_F_LINE_10",
        "Form 2210-F line 10: your 2025 tax at 100%, with your spouse's separate 2025 tax when you now file jointly; the 110% rule for higher incomes does not apply to farmers and fishers.",
    ),
    (
        "EXPLAIN_FORM_2210_F_LINE_11",
        "Form 2210-F line 11: required annual payment, the smaller of line 7 and line 10, or line 7 when your 2025 return did not cover 12 months.",
    ),
    (
        "EXPLAIN_FORM_2210_F_LINE_12",
        "Form 2210-F line 12: estimated tax paid by January 15, 2027, plus all 2026 withholding.",
    ),
    ("EXPLAIN_FORM_2210_F_LINE_13", "Form 2210-F line 13: underpayment, line 11 minus line 12."),
    (
        "EXPLAIN_FORM_2210_F_LINE_16",
        "Form 2210-F line 16: line 13 times the days from January 15, 2027 until it was paid (each later payment on its date, the rest April 15, 2027), over 365, times the IRS underpayment rate for the first quarter of 2027. While that rate is unpublished, the last published rate is used.",
    ),
    (
        "EXPLAIN_FORM_2210_F_LINE_2",
        "Form 2210-F line 2: the Schedule 2 taxes the instructions list, including household employment tax unless line 25d is zero and the penalty is zero without it.",
    ),
    ("EXPLAIN_FORM_2210_F_LINE_3", "Form 2210-F line 3: line 1 plus line 2."),
    (
        "EXPLAIN_FORM_2210_F_LINE_4",
        "Form 2210-F line 4: refundable credits (earned income, additional child, refundable American opportunity and adoption credits, net premium tax credit, fuel tax credit and section 1341 credit).",
    ),
    ("EXPLAIN_FORM_2210_F_LINE_6", "Form 2210-F line 6: current year tax, line 3 minus line 4."),
    ("EXPLAIN_FORM_2210_F_LINE_7", "Form 2210-F line 7: 66⅔% of line 6."),
    (
        "EXPLAIN_FORM_2210_F_LINE_8",
        "Form 2210-F line 8: Form 1040 line 25d plus excess Social Security and tier 1 railroad retirement tax.",
    ),
    ("EXPLAIN_FORM_2210_F_LINE_9", "Form 2210-F line 9: line 6 minus line 8."),
    ("EXPLAIN_FORM_2210_F_NO_PENALTY_11", "Line 8 is at least line 11: no penalty."),
    ("EXPLAIN_FORM_2210_F_NO_PENALTY_13", "Line 13 is zero or less: no penalty."),
    ("EXPLAIN_FORM_2210_F_NO_PENALTY_3", "Line 3 is less than $1,000: no penalty."),
    ("EXPLAIN_FORM_2210_F_NO_PENALTY_6", "Line 6 is less than $1,000: no penalty."),
    ("EXPLAIN_FORM_2210_F_NO_PENALTY_9", "Line 9 is less than $1,000: no penalty."),
    (
        "EXPLAIN_FORM_2210_F_PENALTY_BEFORE_WAIVER",
        "Form 2210-F: the penalty figured without regard to the waiver; line 16 is this less the amount waived.",
    ),
    (
        "EXPLAIN_FORM_2210_F_WAIVER_AMOUNT",
        "Form 2210-F: the amount you ask to be waived, entered in parentheses beside line 16; the IRS decides the waiver; the penalty after the amount you ask to waive is shown, and the computed penalty is its own row.",
    ),
    (
        "EXPLAIN_FORM_2210_LINE_10_2",
        "Form 2210 line 10: required installment from Schedule AI line 27 (box C).",
    ),
    ("EXPLAIN_FORM_2210_LINE_10_3", "Form 2210 line 10: required installment, 25% of line 9."),
    (
        "EXPLAIN_FORM_2210_LINE_11",
        "Form 2210 line 11: withholding treated as paid on the due date plus estimated payments made in this period.",
    ),
    (
        "EXPLAIN_FORM_2210_LINE_17",
        "Form 2210 line 17: underpayment for this due date after earlier payments cover earlier installments first.",
    ),
    (
        "EXPLAIN_FORM_2210_NO_PENALTY",
        "Farmers and fishermen: no penalty applies because your 2026 return was filed and the tax shown on it was paid in full by March 1, 2027.",
    ),
    ("EXPLAIN_FORM_2210_NO_PENALTY_2", "Line 4 is less than $1,000: no penalty."),
    ("EXPLAIN_FORM_2210_NO_PENALTY_3", "Line 7 is less than $1,000: no penalty."),
    ("EXPLAIN_FORM_2210_NO_PENALTY_4", "Line 9 is not more than line 6: no penalty."),
    (
        "EXPLAIN_FORM_2210_PART_1",
        "Form 2210 line 1: Form 1040 line 22, tax after nonrefundable credits.",
    ),
    (
        "EXPLAIN_FORM_2210_PART_1_10",
        "Form 2210 line 9: required annual payment, the smaller of line 5 and line 8.",
    ),
    (
        "EXPLAIN_FORM_2210_PART_1_2",
        "Form 2210 line 2: the Schedule 2 taxes the Form 2210 instructions list, including household employment tax unless line 25d is zero and the penalty is zero without it.",
    ),
    (
        "EXPLAIN_FORM_2210_PART_1_3",
        "Form 2210 line 3: refundable credits (earned income, additional child, refundable American opportunity and adoption credits, net premium tax credit, fuel tax credit and section 1341 credit).",
    ),
    (
        "EXPLAIN_FORM_2210_PART_1_4",
        "Form 2210 line 4: current year tax, line 1 plus line 2 minus line 3.",
    ),
    ("EXPLAIN_FORM_2210_PART_1_6", "Form 2210 line 5: 90% of line 4."),
    (
        "EXPLAIN_FORM_2210_PART_1_7",
        "Form 2210 line 6: federal income tax withheld (Form 1040 line 25d) plus excess Social Security and tier 1 railroad retirement tax.",
    ),
    ("EXPLAIN_FORM_2210_PART_1_8", "Form 2210 line 7: line 4 minus line 6."),
    (
        "EXPLAIN_FORM_2210_PART_1_9",
        "Form 2210 line 8: your 2025 tax, at 110% when 2025 AGI was over $150,000 ($75,000 married filing separately); a joint filer whose 2025 status differed adds the spouse's separately entered 2025 tax (box E).",
    ),
    (
        "EXPLAIN_FORM_2441",
        "Earned-income limit for dependent care, including applicable monthly deemed earnings. Deemed amounts do not enter taxable income, IRA compensation or EIC earnings.",
    ),
    (
        "EXPLAIN_FORM_N_CORRECTED_INCOME_TAX",
        "Form 8978 lines 6 and 11: corrected 2025 income tax using the published Tax Table.",
    ),
    (
        "EXPLAIN_FORM_N_INJURED_CONTRIBUTION",
        "The injured spouse's withholding, payments, refundable credits and share of the earned income credit.",
    ),
    (
        "EXPLAIN_FORM_N_INJURED_EIC",
        "The injured spouse's part of the earned income credit, by each spouse's separate credit with the joint return's children.",
    ),
    (
        "EXPLAIN_FORM_N_INJURED_LIABILITY",
        "The injured spouse's share of the joint tax less their nonrefundable credits, plus their other taxes.",
    ),
    (
        "WARN_TAX_CREDIT_RIC_ORDINARY",
        "The unreported RIC credit distribution is included as ordinary dividends. To retain an issuer-designated qualified portion, enter the actual distribution and qualified amount on its dividend statement, then link the already-reported income here. No qualified amount is inferred from the credit.",
    ),
    (
        "EXPLAIN_SCHEDULE_B_SOURCE",
        "{0}: {1}. Exempt interest is excluded. These listings reconcile to the retained source amounts; the return total is rounded to whole dollars.",
    ),
    ("LINE_SCHEDULE_B_INTEREST_N_PAYER", "Schedule B interest source {0}: payer"),
    ("LINE_SCHEDULE_B_INTEREST_N_NOMINEE", "Schedule B interest source {0}: nominee"),
    ("LINE_SCHEDULE_B_INTEREST_N_ESTATE", "Schedule B interest source {0}: estate"),
    ("LINE_SCHEDULE_B_INTEREST_N_PRIOR", "Schedule B interest source {0}: prior"),
    ("LINE_SCHEDULE_B_INTEREST_N_ACCRUED", "Schedule B interest source {0}: accrued"),
    ("LINE_SCHEDULE_B_INTEREST_N_ADJUSTMENT", "Schedule B interest source {0}: adjustment"),
    ("LINE_SCHEDULE_B_INTEREST_N_OTHER", "Schedule B interest source {0}: other"),
    ("LINE_SCHEDULE_B_DIVIDEND_N_PAYER", "Schedule B dividend source {0}: payer"),
    ("LINE_SCHEDULE_B_DIVIDEND_N_NOMINEE", "Schedule B dividend source {0}: nominee"),
    ("LINE_SCHEDULE_B_DIVIDEND_N_ESTATE", "Schedule B dividend source {0}: estate"),
    ("LINE_SCHEDULE_B_DIVIDEND_N_PRIOR", "Schedule B dividend source {0}: prior"),
    ("LINE_SCHEDULE_B_DIVIDEND_N_ACCRUED", "Schedule B dividend source {0}: accrued"),
    ("LINE_SCHEDULE_B_DIVIDEND_N_ADJUSTMENT", "Schedule B dividend source {0}: adjustment"),
    ("LINE_SCHEDULE_B_DIVIDEND_N_OTHER", "Schedule B dividend source {0}: other"),
    (
        "DIV_CAPITAL_RETURNS_RECONCILE",
        "Allocate each printed 1099-DIV box 3, 9 and 10 exactly among its actual share lots; include the corresponding basis, holding period and liquidation facts. Do not duplicate these distributions in another document.",
    ),
    ("LINE_DISTRIBUTION_N_LOT_N_BASIS_REDUCTION", "Distribution {0}, lot {1}: basis reduction"),
    ("LINE_DISTRIBUTION_N_LOT_N_CAPITAL_GAIN", "Distribution {0}, lot {1}: capital gain"),
    ("LINE_DISTRIBUTION_N_LOT_N_PROPERTY_N_BASIS", "Distribution {0}, lot {1}: property {2} basis"),
    ("LINE_DISTRIBUTION_N_LOT_N_REMAINING_BASIS", "Distribution {0}, lot {1}: remaining basis"),
    (
        "EXPLAIN_CREDIT_ENTITY_STATEMENTS",
        "Part IV line 20: eligible entity credit amounts reported on statements other than Form 1097-BTC. A duplicate Form 1097-BTC credit is entered only once in Part III.",
    ),
    (
        "EXPLAIN_CREDIT_SOURCE_ROW",
        "{0}, bond {1}: Form 8912 Part {2}. This is the eligible current credit before the return-level tax limit.",
    ),
    ("LINE_TAX_CREDIT_BOND_III_N", "Form 8912 Part III source {0}"),
    ("LINE_TAX_CREDIT_BOND_IV_N", "Form 8912 Part IV source {0}"),
    (
        "EXPLAIN_FORM_N_LINE1",
        "Part III line 14: eligible direct or entity bond credits copied from annual Forms 1097-BTC box 1, each form rounded once for its line 13 entry.",
    ),
    (
        "EXPLAIN_FORM_N_LINE11",
        "Part II line 11: regular income tax including Schedule 2 line 1z, plus AMT, less foreign tax, other allowable credits, the allowed Form 3800 credit, and the prior-year minimum-tax credit. The allowed Schedule 3 line 6l Form 8978 credit also reduces this limit.",
    ),
    (
        "EXPLAIN_FORM_N_LINE3",
        "Part I line 3: unused prior-year qualified tax-credit or holder-credit Build America bond credit, tied to each same bond; it is not taxable interest again.",
    ),
    (
        "EXPLAIN_FORM_N_LINE4",
        "Part I line 4: current bond credit plus eligible same-bond prior carry.",
    ),
    (
        "EXPLAIN_FORM_N_LINE_N",
        "Signed Form 8978 line 14: corrected income tax liability less the previously reported liability.",
    ),
    ("EXPLAIN_FORM_N_LINE_N_12", "Student {0}: {1}."),
    (
        "EXPLAIN_FORM_N_LINE_N_2",
        "Form 2210 line 19: penalty worksheet total: each unpaid amount times the days it stayed unpaid, through April 15, 2027, over 365, times the IRS quarterly underpayment rate. A quarter the IRS has not yet published uses the last published rate.",
    ),
    ("EXPLAIN_FORM_N_LINE_N_3", "The tax year for which Form 8862 is filed."),
    ("EXPLAIN_FORM_N_LINE_N_6", "Child {0}: {1}."),
    ("EXPLAIN_FORM_N_LINE_N_7", "Yes: Schedule EIC for 2026 shows a qualifying child."),
    (
        "EXPLAIN_FORM_N_LINE_N_CHILD_N_BIRTH",
        "{0} was born during 2026: the month and day of birth as MMDD.",
    ),
    (
        "EXPLAIN_FORM_N_LINE_N_CHILD_N_DEATH",
        "{0} died during 2026: the month and day of death as MMDD.",
    ),
    (
        "EXPLAIN_FORM_N_LINE_N_INTEREST",
        "Interest on the positive Form 8978 tax adjustment from the separate supporting computation. It belongs in the reporting-year net payment, not on Form 1040.",
    ),
    (
        "EXPLAIN_FORM_N_LINE_N_N_2",
        "Yes: {0} lived with you for more than half of 2026 ({1} days) or meets an exception.",
    ),
    (
        "EXPLAIN_FORM_N_LINE_N_N_3",
        "Yes: {0} meets the relationship, age, support, residency, dependent, joint-return and citizenship tests of a CTC/ACTC qualifying child.",
    ),
    ("EXPLAIN_FORM_N_LINE_N_N_4", "Yes: {0} is your dependent on this return."),
    (
        "EXPLAIN_FORM_N_LINE_N_N_5",
        "Yes: {0} is a citizen, national or resident of the United States.",
    ),
    (
        "EXPLAIN_FORM_N_NET_PAYMENT_AFTER_INTEREST",
        "Reporting-year net payment after Form 8978 interest: positive means amount due, negative means refund remaining. Form 1040 balance and refund exclude this separate interest.",
    ),
    (
        "EXPLAIN_FORM_N_N_N_N_EXCESS_REPAYMENT",
        "Repayments above this year's income: carry them to your 2027 Form 8915-F (Worksheet 3 or 5 line 1), or back by amending an earlier return.",
    ),
    (
        "EXPLAIN_FORM_N_N_N_N_INSTALLMENT_N",
        "The 2027 line 11 and line 22 amounts of this year's spread distributions, carried to your 2027 Form 8915-F.",
    ),
    (
        "EXPLAIN_FORM_N_N_N_N_INSTALLMENT_N_2",
        "The 2028 line 11 and line 22 amounts of this year's spread distributions.",
    ),
    (
        "EXPLAIN_FORM_N_N_N_N_LINE_N",
        "Part I line 6: 2026 qualified disaster distributions, each within its disaster's $22,000 less earlier years. No additional tax on early distributions applies to them.",
    ),
    (
        "EXPLAIN_FORM_N_N_N_N_LINE_N_10",
        "Lines 18 and 20: taxable amount of IRA qualified disaster distributions (the Form 8606 share when it applies).",
    ),
    (
        "EXPLAIN_FORM_N_N_N_N_LINE_N_11",
        "Line 22: one third of line 21, or all of it when you chose to include it all in 2026.",
    ),
    (
        "EXPLAIN_FORM_N_N_N_N_LINE_N_12",
        "Line 23 (Worksheet 4): line 22 of your 2024 and 2025 Forms 8915-F for these disasters.",
    ),
    ("EXPLAIN_FORM_N_N_N_N_LINE_N_13", "Line 24: lines 22 plus 23."),
    (
        "EXPLAIN_FORM_N_N_N_N_LINE_N_14",
        "Line 25 (Worksheet 5): excess repayment carried from 2025 plus repayments made for 2026 (taxable part).",
    ),
    ("EXPLAIN_FORM_N_N_N_N_LINE_N_15", "Line 26: IRA amount on Form 1040 line 4b."),
    (
        "EXPLAIN_FORM_N_N_N_N_LINE_N_16",
        "Line 28: distributions received to buy or build a main home in the disaster area.",
    ),
    (
        "EXPLAIN_FORM_N_N_N_N_LINE_N_17",
        "Line 29: the part of those distributions that is your cost (not taxable).",
    ),
    ("EXPLAIN_FORM_N_N_N_N_LINE_N_18", "Line 30: line 28 less line 29, the taxable part."),
    (
        "EXPLAIN_FORM_N_N_N_N_LINE_N_19",
        "Line 31: the taxable part repaid to a retirement plan or IRA by the end of the repayment period.",
    ),
    (
        "EXPLAIN_FORM_N_N_N_N_LINE_N_20",
        "Line 32: line 30 less line 31, included in income (Form 1040 line 4b or 5b) and possibly subject to the additional tax.",
    ),
    (
        "EXPLAIN_FORM_N_N_N_N_LINE_N_2",
        "Part II line 8: qualified disaster distributions from employer plans.",
    ),
    (
        "EXPLAIN_FORM_N_N_N_N_LINE_N_3",
        "Line 9: their cost, the designated share of each 1099-R's nontaxable amount.",
    ),
    (
        "EXPLAIN_FORM_N_N_N_N_LINE_N_4",
        "Line 10: taxable amount of employer-plan qualified disaster distributions.",
    ),
    (
        "EXPLAIN_FORM_N_N_N_N_LINE_N_5",
        "Line 11: one third of line 10, or all of it when you chose to include it all in 2026.",
    ),
    (
        "EXPLAIN_FORM_N_N_N_N_LINE_N_6",
        "Line 12 (Worksheet 2): line 11 of your 2024 and 2025 Forms 8915-F for these disasters.",
    ),
    ("EXPLAIN_FORM_N_N_N_N_LINE_N_7", "Line 13: lines 11 plus 12."),
    (
        "EXPLAIN_FORM_N_N_N_N_LINE_N_8",
        "Line 14 (Worksheet 3): excess repayment carried from 2025 plus repayments made for 2026 (taxable part).",
    ),
    ("EXPLAIN_FORM_N_N_N_N_LINE_N_9", "Line 15: employer-plan amount on Form 1040 line 5b."),
    (
        "EXPLAIN_FORM_N_PERSONAL_GAINS",
        "Personal-use casualty gains from reimbursement above basis. They offset eligible undeclared losses first, then ordinary and qualified declared-disaster losses; recognized excess enters Schedule D.",
    ),
    (
        "EXPLAIN_FORM_N_PERSONAL_LOSS_AFTER_AGI",
        "Provisional 2026 Form 4684 line 18: ordinary personal-use disaster loss after its 10%-of-AGI reduction. Claimed only on the itemized route.",
    ),
    (
        "EXPLAIN_FORM_N_PERSONAL_LOSS_AFTER_EVENTS",
        "Form 4684 line 14: declared-disaster losses after $100 per ordinary event or $500 per qualified major-disaster event, plus undeclared losses after $100 per event only up to current personal casualty gains. Before gain and AGI reductions.",
    ),
    (
        "EXPLAIN_FORM_N_PERSONAL_NET_AFTER_GAINS",
        "Personal-use disaster losses after event reductions and casualty gains, before the ordinary-loss 10%-of-AGI reduction. Qualified major-disaster loss bypasses that AGI reduction.",
    ),
    (
        "EXPLAIN_FORM_N_PRIOR_HOUSEHOLD_N_COST",
        "Original purchase cost recorded with the 2025 Form 8283 Section A and valuation evidence; the 50%-class contribution was limited to the lower used-item fair market value.",
    ),
    (
        "EXPLAIN_FORM_N_PRIOR_HOUSEHOLD_N_VALUE",
        "Original 2025 Form 8283 Section A fair market value of the personally purchased furnishing. It is evidence for the carryover, not another 2026 contribution.",
    ),
    (
        "EXPLAIN_FORM_N_PRIOR_INCOME_TAX",
        "Form 8978 line 12: verified 2025 ordinary income tax before the BBA-audit portfolio-interest change.",
    ),
    (
        "EXPLAIN_FORM_N_SECTION_A_VALUE",
        "Publicly traded stock at FMV or elected reduced basis, personally purchased clothing, household goods, books or jewelry in similar-property groups of $5,000 or less, and qualifying Section A vehicle claims, reportable in Form 8283 Section A before income limits and the 2026 charitable floor. Public stock stays in Section A without an appraisal even above $5,000. Attach required acknowledgments and the form to a filed return; this on-screen estimate does not prepare the attachment.",
    ),
    (
        "EXPLAIN_FORM_N_SECTION_B_VALUE",
        "Personally purchased clothing, household goods, books or jewelry in similar-property groups over $5,000, plus FMV-based road-vehicle exception claims whose similar-vehicle group exceeds $5,000, before income limits and the 2026 charitable floor, reportable in Form 8283 Section B. Gross-proceeds-limited vehicles stay in Section A. Each affected gift needs a qualified appraisal and each donee needs the required appraisal summary and signatures; this on-screen estimate does not prepare or attach the form.",
    ),
    (
        "EXPLAIN_FORM_N_UNDECLARED_LOSS_AFTER_EVENTS",
        "Personal-use casualty losses outside a federal or state-declared disaster after the $100-per-event reduction. Form 4684 line 14 uses no more than current personal casualty gains; unused loss is not deductible.",
    ),
    (
        "EXPLAIN_FORM_N_UNUSABLE_DECREASE",
        "Negative Form 8978 amount remaining after the Schedule 3 and Schedule 2 limits. It does not increase a refund or reduce other additional taxes.",
    ),
    (
        "EXPLAIN_FORM_N_PENALTY_BEFORE_WAIVER",
        "Form 2210: the penalty figured through line 18 without regard to the waiver; line 19 is this less the amount waived.",
    ),
    (
        "EXPLAIN_FORM_N_WAIVER_REQUESTED_AMOUNT",
        "Form 2210 Part II box B: amount of the penalty requested to be waived (at most the penalty), subtracted from the penalty to give line 19. The IRS decides the waiver; attach your explanation and documentation.",
    ),
    (
        "EXPLAIN_FUEL_TAX_CREDIT",
        "2026 Form 4136 line 17 and Schedule 3 line 12: supported direct-business, partnership K-1 and home-kerosene fuel tax credits, reported as one payment. Business income timing depends on expense treatment and accounting method; qualifying personal home kerosene is a payment only.",
    ),
    (
        "EXPLAIN_FUND_N_BASIS_INCREASE",
        "Increase this fund's share basis by the reported gain less the tax-paid credit. Retain with share records and include once in any later sale basis; this is not additional income or a deduction.",
    ),
    (
        "EXPLAIN_GAMBLING_LOSS_DEDUCTION",
        "2026 itemized deduction: the smaller of gambling winnings and 90% of documented losses. No separate deduction under the standard-deduction route.",
    ),
    (
        "EXPLAIN_GAMBLING_WINNINGS",
        "Casual gambling winnings enter Schedule 1 in full, without netting itemized losses against income.",
    ),
    (
        "EXPLAIN_GENERAL_BUSINESS_CREDIT",
        "2026 Form 3800 Part II line 38 and Schedule 3 line 6a: allowed general business credit, oldest year first.",
    ),
    (
        "EXPLAIN_GENERAL_BUSINESS_CREDIT_10C",
        "Part II lines 10a–10c: foreign tax credit and the other allowable credits of Form 1040 line 19 and Schedule 3 lines 2–4, 5a and 7.",
    ),
    (
        "EXPLAIN_GENERAL_BUSINESS_CREDIT_LIMIT",
        "2026 Form 3800 Part II lines 27–29: net income tax including AMT, less 25% of net regular tax over $25,000 and the credits allowed on line 28.",
    ),
    (
        "EXPLAIN_GENERAL_BUSINESS_CREDIT_N",
        "Part I line 1: current nonpassive credits not allowed against tentative minimum tax (Part III line 2, column (e)).",
    ),
    ("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_10", "Part II line 12: net regular tax."),
    (
        "EXPLAIN_GENERAL_BUSINESS_CREDIT_N_11",
        "Part II line 13: 25% of net regular tax over $25,000 ($12,500 married filing separately unless the spouse has no business credit).",
    ),
    (
        "EXPLAIN_GENERAL_BUSINESS_CREDIT_N_12",
        "Part II line 14: Form 6251 line 9 tentative minimum tax.",
    ),
    ("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_13", "Part II line 15: the greater of lines 13 and 14."),
    ("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_14", "Part II line 16: line 11 less line 15."),
    (
        "EXPLAIN_GENERAL_BUSINESS_CREDIT_N_15",
        "Part II line 17: credit allowed after the section 38(c)(1) limit.",
    ),
    (
        "EXPLAIN_GENERAL_BUSINESS_CREDIT_N_16",
        "Part II line 26: empowerment zone employment credit; none on this return.",
    ),
    ("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_17", "Part II line 27: line 11 less line 13."),
    ("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_18", "Part II line 28: lines 17 and 26."),
    ("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_19", "Part II line 29: room for specified credits."),
    (
        "EXPLAIN_GENERAL_BUSINESS_CREDIT_N_2",
        "Part I line 2: passive credits not allowed against tentative minimum tax, current and carried, before Form 8582-CR.",
    ),
    (
        "EXPLAIN_GENERAL_BUSINESS_CREDIT_N_20",
        "Part II line 30: current nonpassive specified credits (Part III line 5, column (e)).",
    ),
    (
        "EXPLAIN_GENERAL_BUSINESS_CREDIT_N_21",
        "Part II line 32: passive specified credits, current and carried, before Form 8582-CR.",
    ),
    (
        "EXPLAIN_GENERAL_BUSINESS_CREDIT_N_22",
        "Part II line 33: passive specified credits allowed by Form 8582-CR (line 37).",
    ),
    (
        "EXPLAIN_GENERAL_BUSINESS_CREDIT_N_23",
        "Part II line 34: nonpassive specified carryforwards to 2026 (Part IV line 5, column (f)).",
    ),
    ("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_24", "Part II line 36: lines 30, 33 and 34."),
    ("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_25", "Part II line 37: specified credits allowed."),
    (
        "EXPLAIN_GENERAL_BUSINESS_CREDIT_N_26",
        "Part II line 38 and Schedule 3 line 6a: general business credit allowed for 2026.",
    ),
    (
        "EXPLAIN_GENERAL_BUSINESS_CREDIT_N_3",
        "Part I line 3: passive credits allowed by Form 8582-CR (line 37).",
    ),
    (
        "EXPLAIN_GENERAL_BUSINESS_CREDIT_N_4",
        "Part I line 4: nonpassive carryforwards to 2026 of credits not allowed against tentative minimum tax (Part IV line 6, column (f)).",
    ),
    ("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_5", "Part I line 6: lines 1, 3 and 4."),
    (
        "EXPLAIN_GENERAL_BUSINESS_CREDIT_N_6",
        "Part II line 7: Form 1040 line 16 and Schedule 2 line 1z.",
    ),
    (
        "EXPLAIN_GENERAL_BUSINESS_CREDIT_N_7",
        "Part II line 8: Form 6251 line 11 alternative minimum tax.",
    ),
    ("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_8", "Part II line 9: lines 7 and 8."),
    ("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_9", "Part II line 11: net income tax."),
    (
        "EXPLAIN_GROSS_INCOME_N",
        "Total gross income for the farmer and fisherman estimated-tax test: the return's income (not loss) amounts, with each business's gross income in place of its net profit. The test is met when farming and fishing is at least two-thirds of this.",
    ),
    (
        "EXPLAIN_HEALTH_LOAN_INTEREST_REMOVED",
        "Health-service program-paid interest removed from the student-loan interest claim before the cap and phaseout.",
    ),
    (
        "EXPLAIN_HEALTH_LOAN_N_EXCLUDED",
        "Qualifying health-service loan repayment assistance excluded from income. Program-paid interest cannot also be deducted; disclosed overlap is removed before the student-loan deduction cap and phaseout.",
    ),
    (
        "EXPLAIN_HOBBY_SALES_INCOME",
        "Ordinary hobby goods sales less calculated cost of goods sold, floored at zero, on Schedule 1 line 8j. No operating-expense deduction, business loss, QBI or self-employment tax. Property-sale gain remains subject to applicable net investment income tax.",
    ),
    (
        "EXPLAIN_HOME_KEROSENE_FUEL_4A",
        "Form 4136 line 4a, type of use 8: home-use undyed kerosene at 24.3 cents per gallon. This is the business-information exception, a payment credit and not taxable business income.",
    ),
    (
        "EXPLAIN_HOME_N_N_DEPRECIATION_GAIN",
        "Post-May 6, 1997 depreciation cannot be excluded. This component enters the shared section 1250 gain worksheet, within the total taxable gain, without a second basis reduction.",
    ),
    (
        "EXPLAIN_HOME_N_N_EXCLUDED_GAIN",
        "Section 121 exclusion from ownership, residence and prior-sale facts; excluded gain does not enter AGI or net investment income.",
    ),
    (
        "EXPLAIN_HOME_N_N_NONQUALIFIED_GAIN",
        "Gain allocated by nonqualified days divided by total ownership days, after removing the depreciation component. This amount cannot be excluded.",
    ),
    (
        "EXPLAIN_HOME_N_N_REALIZED_GAIN",
        "Selling price less selling expenses and adjusted basis; personal losses are not deductible.",
    ),
    (
        "EXPLAIN_HOME_N_N_SEPARATE_PRICE",
        "The separate business or rental space's share of the selling price, by floor area (Pub. 523, Business Use or Rental of Home).",
    ),
    (
        "EXPLAIN_HOME_N_N_SEPARATE_EXPENSES",
        "The separate space's share of the selling expenses, by floor area.",
    ),
    (
        "EXPLAIN_HOME_N_N_SEPARATE_BASIS",
        "The separate space's share of the cost, by floor area, less the depreciation taken on it. The rest of the price, expenses and basis is your home's sale.",
    ),
    (
        "EXPLAIN_HOME_N_N_SEPARATE_GAIN",
        "Gain or loss on the separate space, a section 1231 item on Form 4797 line 2 that the home-sale exclusion does not reach.",
    ),
    (
        "EXPLAIN_HOME_N_N_SEPARATE_UNRECAPTURED",
        "The straight-line depreciation on the separate space, up to its gain: unrecaptured section 1250 gain taxed at up to 25% (Schedule D Tax Worksheet).",
    ),
    (
        "EXPLAIN_HOME_N_N_FIRPTA_WITHHOLDING",
        "Tax the buyer withheld from a foreign seller's price and sent in with Form 8288-A, claimed as withholding on Form 1040 line 25c.",
    ),
    (
        "EXPLAIN_HOME_N_N_INSTALLMENT_EXCLUSION",
        "The section 121 exclusion, subtracted from the gross profit on line 14 before the gross profit percentage (Pub. 523, Installment Sale).",
    ),
    (
        "EXPLAIN_HOME_N_N_EXCHANGE_LINE15",
        "Form 8824 line 15: the cash and fair market value of other property received, plus loans the other party took over, less your exchange expenses (not below zero).",
    ),
    (
        "EXPLAIN_HOME_N_N_EXCHANGE_LINE16",
        "Form 8824 line 16: the fair market value of the like-kind property received.",
    ),
    (
        "EXPLAIN_HOME_N_N_EXCHANGE_LINE18",
        "Form 8824 line 18: the old home's adjusted basis plus any exchange expenses not used on line 15.",
    ),
    (
        "EXPLAIN_HOME_N_N_EXCHANGE_LINE19",
        "Form 8824 line 19: the realized gain, lines 15 and 16 less line 18. The home exclusion does not reduce it.",
    ),
    (
        "EXPLAIN_HOME_N_N_EXCHANGE_LINE20",
        "Form 8824 line 20: the smaller of line 15 less the home exclusion (Rev. Proc. 2005-14) or line 19, not below zero: the recognized gain.",
    ),
    (
        "EXPLAIN_HOME_N_N_EXCHANGE_LINE24",
        "Form 8824 line 24: the gain deferred, line 19 less the recognized gain.",
    ),
    (
        "EXPLAIN_HOME_N_N_EXCHANGE_LINE25",
        "Form 8824 line 25: the basis of the new property, line 18 plus the recognized gain, less line 15, plus the home exclusion. Use it as the new property's basis.",
    ),
    (
        "EXPLAIN_HOME_N_N_TAXABLE_GAIN",
        "Home-sale gain remaining after exclusion, included once in capital-gain netting.",
    ),
    (
        "EXPLAIN_HOUSEHOLD_N_N",
        "Schedule H employer liability, including employee taxes collected. Not the employer's income, personal withholding credit or self-employment tax.",
    ),
    (
        "EXPLAIN_HSA_ADDITIONAL_TAX",
        "20% of nonmedical withdrawals on or before the 65th birthday, except withdrawals after permanent and total disability.",
    ),
    (
        "EXPLAIN_HSA_CORRECTION_EARNINGS",
        "Positive attributable earnings on timely returned excess contributions enter other income in the withdrawal year, without ordinary HSA withdrawal tax. Corrected employer principal already in W-2 wages is not added again.",
    ),
    (
        "EXPLAIN_HSA_CORRECTION_EMPLOYER_PRINCIPAL_OTHER_INCOME",
        "Timely returned employer HSA excess principal actually deposited in 2026 and omitted from W-2 wages enters 2026 Schedule 1 other income, even if designated for 2025 or returned in 2027. Corrected principal already in 2026 W-2 wages is not counted again. A 2027 deposit follows 2027 income reporting. This other income is not EIC earnings or IRA compensation.",
    ),
    (
        "EXPLAIN_HSA_FUNDING_BASIS_N_N",
        "Taxable IRA money is used first. This transfer amount consumes basis, removed from the source IRA and not carried into the HSA; later withdrawals use the remaining basis.",
    ),
    (
        "EXPLAIN_HSA_FUNDING_N_N",
        "Qualified direct IRA funding: excluded from IRA income and early-distribution tax, not deductible, and reduces other HSA contribution capacity. Keep eligibility records through the same month of 2027; a nonexcepted loss in 2026 is recaptured here.",
    ),
    (
        "EXPLAIN_HSA_FUNDING_RECAPTURE",
        "Prior qualified IRA-to-HSA funding included in 2026 income when eligibility first fails in its testing period, except loss caused by death or disability.",
    ),
    (
        "EXPLAIN_HSA_LAST_MONTH_RECAPTURE",
        "Last-month-rule recapture for a first loss in 2026, including prior-year and December 2026 contributions. Income is capped at non-IRA contributions; qualified IRA funding retains its own testing period and is not included twice.",
    ),
    (
        "EXPLAIN_HSA_N_CORRECTED_CONTRIBUTIONS",
        "Timely corrected 2026 principal removed before calculating eligible contributions; attributable earnings are separately reported in the withdrawal year.",
    ),
    (
        "EXPLAIN_HSA_N_DEEMED",
        "Terminated accounts use January 1 fair market value and date; loan pledges use dated collateral values. No medical or rollover exclusion applies.",
    ),
    (
        "EXPLAIN_HSA_N_EXCESS_CONTRIBUTION_TAX",
        "Form 5329 Part VII line 49: 6% of the smaller of remaining prior plus current excess or the HSA value at year end including later contributions designated for 2026. This reaches Schedule 2 line 18 and is not a taxable HSA distribution.",
    ),
    (
        "EXPLAIN_HSA_N_LAST_MONTH_CONTRIBUTIONS",
        "2026 contributions above the ordinary monthly limit; continued eligibility through December 31, 2027 is required unless loss is caused by death or disability. This rule does not backdate account establishment for medical reimbursements.",
    ),
    (
        "EXPLAIN_HSA_N_LATER_EMPLOYER_EXCESS_INCOME",
        "Part of the 2026-designated Form 5329 excess came from employer deposits actually made in 2027. Their wage or other-income inclusion belongs to 2027, not this on-screen 2026 return.",
    ),
    (
        "EXPLAIN_HSA_N_LATE_EXCESS_WITHDRAWALS",
        "Prior-year excess withdrawn after its correction deadline is taxable on Form 8889 line 16 even if spent on medical care; it also reduces retained prior excess through Form 5329 line 44.",
    ),
    (
        "EXPLAIN_HSA_N_N_EXCESS_ACCOUNT_VALUE",
        "All HSA fair market values on December 31, 2026 plus contributions made later and designated for 2026; Form 5329 limits the 6% excise tax to this value.",
    ),
    (
        "EXPLAIN_HSA_N_N_UNCORRECTED_EMPLOYER_EXCESS",
        "Employer contributions above the owner's current Form 8889 limit remain in Form 5329 line 47. The part deposited in 2026 and not already included in W-2 wages enters 2026 other income; a 2027 deposit follows 2027 income reporting.",
    ),
    (
        "EXPLAIN_HSA_N_N_UNCORRECTED_EXCESS",
        "2026 personal contributions above the available Form 8889 limit remain nondeductible and carry into Form 5329 Part VII. The same original-year excess remains relevant to 2027 unless corrected or absorbed under later-year rules.",
    ),
    (
        "EXPLAIN_HSA_N_PRIOR_DESIGNATED_EMPLOYER_EXCESS_INCOME",
        "Uncorrected 2025-designated employer excess actually deposited in 2026 and omitted from W-2 wages enters 2026 other income, not the 2026 Form 8889 contribution total.",
    ),
    (
        "EXPLAIN_HSA_N_PRIOR_EXCESS_N_DEDUCTED_N",
        "Remaining prior excess treated as a 2026 HSA contribution and deducted on Form 8889 line 13 within unused line 12 room.",
    ),
    (
        "EXPLAIN_HSA_N_PRIOR_EXCESS_N_REMAINING_N",
        "Prior excess remaining after taxable HSA distributions and unused 2026 contribution room; combined with current excess for Form 5329 line 48.",
    ),
    (
        "EXPLAIN_HSA_N_PRIOR_EXCESS_N_USED_BY_DISTRIBUTIONS",
        "Retained 2025 Form 5329 line 48 excess absorbed by taxable 2026 HSA distributions on Form 8889 line 16 before unused contribution room is applied.",
    ),
    (
        "EXPLAIN_HSA_N_PRIOR_MONTHLY_LIMIT",
        "2025 monthly contribution limit without the last-month rule, using historical coverage, age-55 catch-up and Archer contributions.",
    ),
    (
        "EXPLAIN_HSA_N_ROLLOVER",
        "Eligible same-owner indirect HSA rollover: excluded from income, not deductible and not counted against ordinary contribution limits.",
    ),
    (
        "EXPLAIN_HSA_N_UNCORRECTED_EMPLOYER_EXCESS_OTHER_INCOME",
        "Uncorrected employer HSA excess actually deposited in 2026 and omitted from W-2 wages enters 2026 Schedule 1 other income once. Any excess already in W-2 wages remains there; a later deposit follows 2027 income reporting. This other income is not EIC earnings or IRA compensation.",
    ),
    (
        "EXPLAIN_HSA_TAXABLE_DISTRIBUTIONS",
        "Nonmedical withdrawals and taxable inherited HSA values enter Schedule 1 income. Inherited values have no additional withdrawal tax. Contribution eligibility does not limit withdrawals from an existing account.",
    ),
    (
        "EXPLAIN_HSA_TESTING_PERIOD_TAX",
        "Form 8889 Part III: combine last-month and IRA-funding recapture before applying 10%, separately for each owner. This is not the 20% withdrawal tax.",
    ),
    (
        "EXPLAIN_INCOME_TAX",
        "Regular 2026 income tax, positive Form 8978 adjustment, excess advance premium credit repayment and AMT, before nonrefundable credits.",
    ),
    (
        "EXPLAIN_INHERITED_ESTATE_TAX_DEDUCTION",
        "Section 691(c) federal estate-tax deduction on inherited IRA income, before the general itemized-deduction limitation. Does not reduce AGI or create an NIIT deduction for IRA income; remains deductible for AMT when itemizing.",
    ),
    (
        "EXPLAIN_INHERITED_IRA_N_N_BASIS_RECOVERED",
        "Separate beneficiary/decedent Form 8606 allocation using this group's annual distributions and year-end balance. Personal contributions and another deceased owner's basis do not enter this fraction.",
    ),
    (
        "EXPLAIN_INHERITED_IRA_N_N_BASIS_REMAINING",
        "Unrecovered allocated basis stays with this inherited group. It is not a deduction and cannot be transferred to personal IRAs or another deceased owner's accounts.",
    ),
    (
        "EXPLAIN_INHERITED_IRA_N_N_ESTATE_IRD_VALUE_REMAINING",
        "Unused estate-tax IRD value for future taxable distributions. Value used this year is consumed even if the standard deduction is selected; it is not a deduction carryforward or IRA basis.",
    ),
    (
        "EXPLAIN_INHERITED_IRA_N_N_ESTATE_TAX_DEDUCTION",
        "Section 691(c): estate tax attributable to net IRD, allocated by the lesser of this year's taxable inherited IRA income or unused estate-tax value, divided by gross estate-wide IRD value. Available only when itemizing; the general itemized-deduction limitation applies.",
    ),
    (
        "EXPLAIN_INHERITED_IRA_N_N_TAXABLE",
        "Inherited traditional IRA income before the filer-level QCD exclusion, less allocated basis recovery. Qualifying charitable transfers use pretax funds first; the shared QCD worksheet applies the annual limit and deductible-contribution adjustment once. The death exception removes early-distribution tax; it does not exclude ordinary taxable income.",
    ),
    (
        "EXPLAIN_INHERITED_MEDICAL_N_N",
        "Inherited medical account: in the death year, nonspouse beneficiaries report allocated death-date value less qualifying bills paid within one year; estate beneficiaries use the decedent's final return without this offset. No additional withdrawal tax on the death transfer. A designated spouse continues the account without taxable inheritance.",
    ),
    (
        "EXPLAIN_INHERITED_ROTH_N_N",
        "Unrecovered allocated basis retained by this beneficiary's Roth IRAs from this deceased owner; do not combine it with the beneficiary's own Roth accounts.",
    ),
    (
        "EXPLAIN_INHERITED_ROTH_N_N_CONVERSION_RECOVERED",
        "This inherited group's allocated conversion basis recovered before earnings, with no early-conversion recapture tax.",
    ),
    (
        "EXPLAIN_INHERITED_ROTH_N_N_REGULAR_RECOVERED",
        "This inherited group's allocated regular-contribution basis recovered first; personal Roth contributions cannot shelter inherited earnings.",
    ),
    (
        "EXPLAIN_INHERITED_ROTH_N_N_TAXABLE",
        "Original-owner five-tax-year qualification excludes the distribution; otherwise inherited contribution and conversion basis is recovered before earnings. The death exception prevents early-distribution tax, including conversion recapture.",
    ),
    (
        "EXPLAIN_INJURED_SPOUSE",
        "Estimated injured spouse share of the joint refund; the IRS makes the final computation.",
    ),
    (
        "EXPLAIN_INSTALLMENT_N",
        "Selling price, including the buyer's note and the loans the buyer took over, without interest.",
    ),
    (
        "EXPLAIN_INSTALLMENT_PROFIT_BEFORE_EXCLUSION",
        "Line 5 selling price less line 13 adjusted basis, selling expenses and recapture; any home-sale exclusion is applied separately on line 15.",
    ),
    (
        "EXPLAIN_INSTALLMENT_GROSS_PROFIT_RATIO",
        "Line 16 gross profit divided by line 18 contract price, expressed as a decimal.",
    ),
    (
        "EXPLAIN_INSTALLMENT_N_10",
        "Line 14 (line 5 less line 13) less the home-sale exclusion on line 15, which is zero unless the property was your main home.",
    ),
    (
        "EXPLAIN_INSTALLMENT_N_11",
        "Loans the buyer took over above line 13, treated as a 2026 payment.",
    ),
    (
        "EXPLAIN_INSTALLMENT_N_12",
        "Line 7 plus line 17. The gross profit percentage (line 19) is line 16 divided by line 18.",
    ),
    ("EXPLAIN_INSTALLMENT_N_13", "Line 17 in the year of sale."),
    ("EXPLAIN_INSTALLMENT_N_14", "Principal received in 2026, without interest."),
    ("EXPLAIN_INSTALLMENT_N_15", "Line 20 plus line 21."),
    (
        "EXPLAIN_INSTALLMENT_N_2",
        "Mortgages and other debts the buyer assumed or took the property subject to.",
    ),
    ("EXPLAIN_INSTALLMENT_N_3", "Line 5 less line 6."),
    ("EXPLAIN_INSTALLMENT_N_4", "Cost or other basis of the property."),
    ("EXPLAIN_INSTALLMENT_N_5", "Depreciation allowed or allowable through the date of sale."),
    ("EXPLAIN_INSTALLMENT_N_6", "Line 8 less line 9."),
    ("EXPLAIN_INSTALLMENT_N_7", "Commissions and other selling expenses."),
    (
        "EXPLAIN_INSTALLMENT_N_8",
        "Section 1245 recapture from Form 4797 line 33, taxed in full in 2026 even without payments.",
    ),
    ("EXPLAIN_INSTALLMENT_N_9", "Lines 10, 11 and 12."),
    (
        "EXPLAIN_INSURANCE_EXCEPTION_N",
        "IRA health-insurance exception based on actual unemployment compensation. Premium relief is limited to the taxable withdrawal; ordinary income remains taxable.",
    ),
    (
        "EXPLAIN_INSURANCE_EXCEPTION_N_2",
        "IRA health-insurance exception based on compensation that would have been received but for self-employed status. No hypothetical unemployment income is added; ordinary IRA income remains taxable.",
    ),
    (
        "EXPLAIN_INVESTMENT_PROPERTY_TAX_DEDUCTION",
        "Itemized real-estate taxes on property held solely for investment appreciation, outside the personal SALT cap; after proportional section 68 reduction. Not deducted again from AGI or capital gain.",
    ),
    (
        "EXPLAIN_IRA_EXCESS_N_N",
        "Form 5329 Parts III and IV: a 2025 excess carried in is reduced by unused 2026 room and 2026 withdrawals; the 2026 excess is the contribution above the App's computed limit. Each part taxes 6% of the smaller of its total excess or the December 31, 2026 value of that IRA type, on Schedule 2 line 18. Line 12 is the part of 2026 withdrawals that paid back an excess from before 2026; it is not income. Line 10 unused room takes in the carried excess (it is a 2026 contribution for the deduction), and a 529-to-Roth transfer above the year's room is a Roth excess on line 23.",
    ),
    (
        "EXPLAIN_ISO_SALE_N_N",
        "ISO source records determine compensation and regular/AMT basis. Compensation already included in W-2 wages is not added again; other compensation enters Schedule 1 line 8k. Same-year sales have no ISO AMT difference. Later-year sales reverse regular compensation for AMT and retain the AMT recognition basis. Restricted shares use vesting facts for regular compensation and independent AMT recognition facts; current-year AMT recognition is included once.",
    ),
    (
        "EXPLAIN_ISO_UNREPORTED_COMPENSATION",
        "Calculated ISO or ESPP compensation not already in W-2 wages enters Schedule 1 line 8k as employee compensation. It is earned income, not self-employment or investment income, and does not increase Social Security or Medicare wages.",
    ),
    (
        "EXPLAIN_ITEMIZED_DEDUCTION_REDUCTION",
        "2026 section 68: 2/37 of the lesser of itemized deductions or income above the 37% threshold after adding back itemized deductions.",
    ),
    (
        "EXPLAIN_JURY_PAY",
        "Jury pay on Schedule 1 line 8h: all jury fees are income; they are not earned income for the EIC or self-employment earnings.",
    ),
    (
        "EXPLAIN_JURY_PAY_REMITTED",
        "Jury pay surrendered to the employer for continued salary is deducted on Schedule 1 line 24a. W-2 wages are unchanged.",
    ),
    (
        "EXPLAIN_EMPLOYEE_WAGES_N_PAY",
        "Form 8919 line 6: employee pay a firm reported on a Form 1099 or paid without payroll tax; it is on Form 1040 line 1g.",
    ),
    (
        "EXPLAIN_EMPLOYEE_WAGES_WAGE_BASE",
        "Form 8919 line 7: the annual Social Security wage base for this return's tax year.",
    ),
    (
        "EXPLAIN_EMPLOYEE_WAGES_USED_BASE",
        "Form 8919 line 8: this employee's W-2 Social Security wages and tips, compensation subject to the 6.2% railroad tax, and Form 4137 covered tips.",
    ),
    (
        "EXPLAIN_EMPLOYEE_WAGES_REMAINING_BASE",
        "Form 8919 line 9: the annual Social Security wage base less line 8, with a minimum of zero.",
    ),
    (
        "EXPLAIN_EMPLOYEE_WAGES_N_SOCIAL_BASE",
        "Form 8919 line 10: the pay under the Social Security wage base left after W-2 wages, covered tips and Form 4137 wages.",
    ),
    (
        "EXPLAIN_EMPLOYEE_WAGES_N_TAX",
        "Form 8919 line 13: the uncollected Social Security and Medicare tax, carried to Schedule 2 line 16b.",
    ),
    (
        "EXPLAIN_KIDDIE_TAX_DIRECT_GAMBLING_DEDUCTION",
        "Casual gambling losses actually deducted after the 2026 loss limit and section 68. Directly connected with ordinary unearned winnings for Form 8615 line 2; no allocation against preferential gains.",
    ),
    (
        "EXPLAIN_KIDDIE_TAX_NET_UNEARNED_INCOME",
        "Form 8615 line 5 is zero after its deduction or taxable-income limit; no parent-rate calculation changes regular tax. This does not determine whether a form must be attached to a filed return.",
    ),
    (
        "EXPLAIN_KIDDIE_TAX_NET_UNEARNED_INCOME_2",
        "Form 8615 line 5: unearned income less the greater of $2,700 or $1,350 plus allowed directly connected deductions, limited to taxable income.",
    ),
    (
        "EXPLAIN_KIDDIE_TAX_PARENT_SHARE",
        "Allocated increase in the parent's tax after including all affected children. The applicable 2026 draft Tax Table is used below $100,000; regular tax is at least the child's own tax.",
    ),
    (
        "EXPLAIN_KIDDIE_TAX_PREFERENTIAL_INTEREST",
        "Allowed investment interest directly connected with qualified dividends or net capital gain, after the itemized-deduction reduction; used in the Form 8615 Line 5 worksheet.",
    ),
    (
        "EXPLAIN_LEGACY_ANNUITY_N_N",
        "Withdrawal ordering: old investment, old earnings, new earnings, new investment. Both earnings portions are income and enter NIIT; earnings allocated to pre-August 14, 1982 investment are exempt from section 72(q) early-distribution tax.",
    ),
    (
        "EXPLAIN_LUMP_SUM_N_LINE_N",
        "A line of Form 4972 for one participant's lump-sum distribution. Part II is the 20% capital gain rate; Part III is the 10-year tax option on the 1986 rate schedule. Amounts are exact; line 30 is rounded to whole dollars.",
    ),
    (
        "EXPLAIN_LUMP_SUM_TAX",
        "Form 4972 line 30 for every participant, included in Form 1040 line 16 (box 2). The amounts taxed on Form 4972 are outside Form 1040 lines 5a and 5b.",
    ),
    (
        "EXPLAIN_LEGACY_EV_PASSIVE_CREDIT",
        "Form 8834 line 7 and Schedule 3 line 6i: current-year Form 8582-CR release, limited by net regular tax above tentative minimum tax.",
    ),
    (
        "EXPLAIN_LEGACY_EV_PASSIVE_CREDIT_UNUSABLE",
        "Form 8834 line 7 instructions: released credit lost to the tax limit cannot carry to another year.",
    ),
    (
        "EXPLAIN_LONG_TERM_CARE_N_N_ELIGIBLE",
        "All entered premiums for this insured person, capped once at the 2026 age limit across personal, business and public-safety policies, after removing tax-free public-safety funding.",
    ),
    (
        "EXPLAIN_LONG_TERM_CARE_N_N_EXCLUDED",
        "Premiums above the age cap or unused child-only premiums that cannot be deducted on Schedule A.",
    ),
    (
        "EXPLAIN_LONG_TERM_CARE_N_N_MEDICAL",
        "Remaining capped medical-eligible long-term-care premiums after self-employed deductions. Added once to medical itemization before the AGI floor.",
    ),
    (
        "EXPLAIN_LTC_BENEFIT_N_EXEMPT",
        "Confirmed qualifying reimbursement or terminal-illness accelerated death benefit excluded from income. Reimbursed care is not also a medical deduction; interest and unqualified policy payments need separate treatment.",
    ),
    (
        "EXPLAIN_LTC_BENEFIT_N_N_N",
        "2026 shared limit: greater of $430 per covered day or qualified care costs, less applicable reimbursements. Insured filing unit first, then proportional allocation to other policyholders; periods calculated separately.",
    ),
    (
        "EXPLAIN_MARKETPLACE",
        "Someone claims you as a dependent: you cannot take the credit and do not file Form 8962. The taxpayer who claims you reconciles the advance credit for your coverage.",
    ),
    (
        "EXPLAIN_MARKETPLACE_ANNUAL_CONTRIBUTION",
        "2026 applicable percentage times household income. Monthly calculations use the rounded annual contribution divided by twelve.",
    ),
    (
        "EXPLAIN_MARKETPLACE_HOUSEHOLD_INCOME",
        "Return MAGI plus required-to-file dependents' MAGI; exempt interest, excluded foreign income and nontaxable Social Security are added back.",
    ),
    (
        "EXPLAIN_MARKETPLACE_POVERTY_LINE",
        "2025 poverty guidelines govern 2026 Marketplace coverage.",
    ),
    (
        "EXPLAIN_MARKETPLACE_SPECIFIED_DEDUCTION",
        "Pub. 974 continuing Step 3/5 worksheet component: specified premiums less their allocable credit, within the existing earned-income limit. Combine any negative component with the same source's selected nonspecified premiums before claiming the nonnegative health deduction. When nonspecified premiums consume the trade limit, retain the separate Worksheet P/Form 7206 stop route.",
    ),
    (
        "EXPLAIN_MARKETPLACE_SPECIFIED_MAXIMUM",
        "Pub. 974 Worksheet X line 30: the smaller of the specified premiums and the business's remaining earned-income limit. 2026 requires full repayment, so no repayment limitation enters.",
    ),
    (
        "EXPLAIN_MARKETPLACE_SPECIFIED_PREMIUMS",
        "Pub. 974 Worksheet W line 1: Marketplace premiums of the months marked under the business, after any Part IV share.",
    ),
    (
        "EXPLAIN_MEDICARE_MSA_N_N",
        "Form 8853 Section B: taxable ordinary and deemed nonmedical distributions; 50% tax after disability/death exceptions and prior balance less 60% of the January 1 deductible. Compute each spouse separately. Age 65 is not an exception.",
    ),
    (
        "EXPLAIN_MINIMUM_TAX_CREDIT",
        "Nonrefundable credit limited to available balance and current tax capacity.",
    ),
    (
        "EXPLAIN_MINIMUM_TAX_CREDIT_AVAILABLE",
        "Prior credit carryforward plus signed newly generated credit, floored at zero.",
    ),
    (
        "EXPLAIN_MINIMUM_TAX_CREDIT_CARRYFORWARD",
        "Unused minimum-tax credit to retain for the next return; not a refundable payment.",
    ),
    (
        "EXPLAIN_MINIMUM_TAX_CREDIT_GENERATED",
        "Prior-year AMT less tax on exclusion items; a negative amount reduces older credit.",
    ),
    (
        "EXPLAIN_MINIMUM_TAX_CREDIT_LIMIT",
        "Current regular liability after other nonrefundable credits, less tentative minimum tax after its foreign credit.",
    ),
    (
        "EXPLAIN_MINIMUM_TAX_EXCLUSION_TAX",
        "Prior-year net minimum tax on permanent exclusion items, using 2025 parameters.",
    ),
    (
        "EXPLAIN_MORTGAGE_AVERAGE_BALANCE",
        "Pub 936 Table 1 line 12: total 2026 average balance of every mortgage on the main and second home.",
    ),
    (
        "EXPLAIN_MORTGAGE_CERTIFIED_INTEREST",
        "Form 8396 line 1: filer-share mortgage interest attributable to certified indebtedness. Whole-loan interest is allocated using certified debt divided by the original mortgage amount, capped at all interest; the ratio stays fixed as principal is repaid.",
    ),
    (
        "EXPLAIN_MORTGAGE_CREDIT_CARRY_N_TO_N",
        "Form 8396 line 16: unused 2024 credit carried to 2027.",
    ),
    (
        "EXPLAIN_MORTGAGE_CREDIT_CARRY_N_TO_N_2",
        "Form 8396 line 14: unused 2025 credit carried to 2027.",
    ),
    (
        "EXPLAIN_MORTGAGE_CREDIT_CARRY_N_TO_N_3",
        "Form 8396 line 17: unused 2026 current credit carried to 2027.",
    ),
    (
        "EXPLAIN_MORTGAGE_CREDIT_CURRENT_POTENTIAL",
        "Form 8396 line 3: filer-share certified interest times certificate rate, calculating each paid period separately when 2026 reissue rates differ, subject to the allocated $2,000 high-rate cap and the original-schedule ceiling for a reissue. This amount reduces itemized mortgage interest even if the credit carries forward.",
    ),
    ("EXPLAIN_MORTGAGE_CREDIT_EXPIRED_N", "Unused 2023 Form 8396 credit expires after 2026."),
    (
        "EXPLAIN_MORTGAGE_INSURANCE_PREMIUMS_8D",
        "Qualified mortgage insurance premiums (Form 1098 box 5) treated as home mortgage interest for 2026, reduced by 10% for each $1,000 ($500 if married filing separately), or part of it, of adjusted gross income over $100,000 ($50,000) and none above $109,000 ($54,500).",
    ),
    (
        "EXPLAIN_MORTGAGE_INTEREST_8A",
        "Home mortgage interest and points reported on Form 1098, limited by the Pub 936 Table 1 qualifying fraction; the Form 8396 mortgage interest credit reduces the interest.",
    ),
    (
        "EXPLAIN_MORTGAGE_INTEREST_8B",
        "Home mortgage interest paid to a seller who financed the home and did not send Form 1098, limited by the same qualifying fraction.",
    ),
    (
        "EXPLAIN_MORTGAGE_INTEREST_8E",
        "Total deductible home mortgage interest, points and mortgage insurance premiums (lines 8a through 8d).",
    ),
    (
        "EXPLAIN_MORTGAGE_INTEREST_CREDIT",
        "Form 8396 line 9: current credit and surviving carries limited by tax after preceding credits and the nonrefundable child-credit reserve.",
    ),
    (
        "EXPLAIN_MORTGAGE_NONDEDUCTIBLE_INTEREST",
        "Pub 936 Table 1 line 16: interest above the qualified loan limit. It is personal interest and not deductible.",
    ),
    (
        "EXPLAIN_MORTGAGE_ORIGINAL_PERIOD_CREDIT",
        "Original-certificate portion of the 2026 Form 8396 line 3 statement: certified interest paid before refinancing times the original credit rate, before the annual and original-schedule ceilings.",
    ),
    (
        "EXPLAIN_MORTGAGE_ORIGINAL_SCHEDULE_CAP",
        "Form 8396 reissue ceiling: 2026 interest scheduled on the original certified mortgage times its original certificate rate, after any allocated $2,000 high-rate cap. The reissued certificate cannot increase line 3 above this amount.",
    ),
    (
        "EXPLAIN_MORTGAGE_POINTS_8C",
        "Points not reported on Form 1098 (a refinance): amortized ratably over the loan term by the months outstanding in 2026, times the Pub 936 Table 1 qualifying fraction. Points reported in Form 1098 box 6 are on line 8a.",
    ),
    (
        "EXPLAIN_MORTGAGE_QUALIFIED_LOAN_LIMIT",
        "Pub 936 Table 1 line 11: grandfathered debt plus October 14, 1987 – December 15, 2017 acquisition debt up to $1,000,000 ($500,000 if married filing separately), then later acquisition debt up to the larger of that result or $750,000 ($375,000).",
    ),
    (
        "EXPLAIN_MORTGAGE_REISSUED_PERIOD_CREDIT",
        "Replacement-certificate portion of the 2026 Form 8396 line 3 statement: certified interest paid after refinancing times the replacement credit rate, before the annual and original-schedule ceilings.",
    ),
    (
        "EXPLAIN_MORTGAGE_SUBSIDY_ADJUSTED_QUALIFYING_INCOME",
        "Form 8828 line 16: issuer-notice income for the family's size at disposition and its full and partial holding years.",
    ),
    (
        "EXPLAIN_MORTGAGE_SUBSIDY_AMOUNT",
        "Form 8828 line 19: 6.25% of this return's share of the highest federally subsidized mortgage principal.",
    ),
    (
        "EXPLAIN_MORTGAGE_SUBSIDY_DISPOSITION_GAIN",
        "Form 8828 line 13: the owner's sale proceeds less selling expenses and adjusted basis, reduced by any qualified subordinate-loan gain share actually paid, or fair market value less basis for a nonspouse gift. Section 121 exclusion does not reduce this line.",
    ),
    (
        "EXPLAIN_MORTGAGE_SUBSIDY_MODIFIED_INCOME",
        "Form 8828 line 15: adjusted gross income plus tax-exempt interest, less gain included in gross income from a matched home sale. A nonspouse gift contributes no sale gain to gross income.",
    ),
    (
        "EXPLAIN_MORTGAGE_SUBSIDY_QSML_PAID_GAIN_SHARE",
        "Form 8828 line 13 qualified subordinate mortgage-loan worksheet: gain share actually paid to the governmental lender, separately from HomeSale selling expenses.",
    ),
    (
        "EXPLAIN_MORTGAGE_SUBSIDY_RECAPTURE_TAX",
        "Total Form 8828 federal mortgage-subsidy recapture tax from this return's subsidized-home dispositions.",
    ),
    (
        "EXPLAIN_MORTGAGE_SUBSIDY_RECAPTURE_TAX_2",
        "Section 143(m)(8)(C): a transfer without recognized gain or loss under section 1041 does not trigger federal mortgage-subsidy recapture or Form 8828.",
    ),
    (
        "EXPLAIN_MORTGAGE_SUBSIDY_RECAPTURE_TAX_3",
        "A qualified home improvement loan that is this home's sole federal mortgage subsidy does not create Form 8828 recapture.",
    ),
    (
        "EXPLAIN_MORTGAGE_SUBSIDY_RECAPTURE_TAX_4",
        "Form 8828 does not apply when the sole original federally subsidized mortgage loan was provided before January 1, 1991.",
    ),
    (
        "EXPLAIN_MORTGAGE_SUBSIDY_RECAPTURE_TAX_5",
        "Form 8828 casualty rule: timely replacement of the destroyed home on its original site for main-home use generally avoids mortgage-subsidy recapture.",
    ),
    (
        "EXPLAIN_MORTGAGE_SUBSIDY_RECAPTURE_TAX_6",
        "No Form 8828 recapture after the original federally subsidized loan's nine-year period.",
    ),
    (
        "EXPLAIN_MORTGAGE_SUBSIDY_RECAPTURE_TAX_7",
        "Form 8828 stops at line 13 when the deemed sale or actual sale produces a loss; the form still has to be filed.",
    ),
    (
        "EXPLAIN_MORTGAGE_SUBSIDY_RECAPTURE_TAX_8",
        "Form 8828 line 23 and Schedule 2 line 13b: lesser of half the disposition gain or the subsidized amount after the issuer or early-repayment holding-period percentage and the income percentage.",
    ),
    ("EXPLAIN_N", "Business income or loss: the Schedule C net profits."),
    (
        "EXPLAIN_NET_INVESTMENT_INCOME_TAX",
        "3.8% of supported net investment income or MAGI excess, whichever is smaller.",
    ),
    (
        "EXPLAIN_NET_INVESTMENT_MODIFIED_AGI",
        "Form 8960 MAGI adds back the earned-income exclusion less its disallowed deductions and includes any beneficiary K-1 box 14 code H MAGI adjustment. Housing exclusion and housing deduction are not added back.",
    ),
    (
        "EXPLAIN_NEW_CLEAN_VEHICLE_CREDIT",
        "Form 8936 line 13 and Schedule 3 line 6f: new clean-vehicle credit after earlier personal and previously owned vehicle credits; unused personal credit expires.",
    ),
    (
        "EXPLAIN_NEW_CLEAN_VEHICLE_CREDIT_UNUSABLE",
        "New personal-use credit lost to the Form 1040 tax limit; it does not carry forward.",
    ),
    (
        "EXPLAIN_NEW_CLEAN_VEHICLE_TRANSFER_REPAYMENT",
        "2026 Schedule 2 line 1b: repay transferred new-vehicle credit when both MAGI years exceed their limits or the vehicle was resold within 30 days.",
    ),
    (
        "EXPLAIN_NIIT_CAPITAL_CARRY_ADJUSTMENT",
        "Excluded capital losses still in the Schedule D carryover, including retained prior differences; retain this Form 8960 worksheet difference for next year's investment-tax calculation.",
    ),
    (
        "EXPLAIN_NIIT_DISPOSITION_ADJUSTMENT",
        "Ordinary property income, non-investment business and realized deferred-NUA exclusions, and prior excluded-loss differences, after the current capital-loss carryover adjustment; add once to capital income in Form 8960.",
    ),
    (
        "EXPLAIN_NIIT_INCOME_TAX_DEDUCTION",
        "Elected state/local income-tax share after the SALT cap and source-income allocation, before the aggregate NIIT section 68 cap. Sales and personal-use property taxes contribute zero.",
    ),
    (
        "EXPLAIN_NOL_N_N_CARRYFORWARD",
        "Unused regular NOL continuing into 2027 after Form 982 reduction, retaining its original loss year.",
    ),
    (
        "EXPLAIN_NOL_N_N_DEBT_REDUCTION",
        "Form 982 reduction after current-year tax, in original loss-year order; not an additional deduction.",
    ),
    (
        "EXPLAIN_NOL_N_N_EXPIRED",
        "Unused pre-2018 balance beyond its twenty-year period, including 2006 losses expiring after 2026.",
    ),
    (
        "EXPLAIN_NOL_N_N_OPENING",
        "Retained regular NOL balance entering 2026; distinct from AMT loss.",
    ),
    (
        "EXPLAIN_NOL_N_N_USED",
        "Loss absorbed by 2026 income, oldest eligible year first. Pre-2018 deduction can exceed absorption.",
    ),
    (
        "EXPLAIN_NONBUSINESS_PERSONAL_RENT",
        "For-profit nonbusiness personal-property gross rent, Schedule 1 line 8l; not self-employment earnings, IRA compensation or QBI.",
    ),
    (
        "EXPLAIN_NONBUSINESS_PERSONAL_RENT_DEPRECIATION",
        "Current bonus and MACRS depreciation before passive loss limits; not a cash operating expense or section 179 deduction.",
    ),
    (
        "EXPLAIN_NONBUSINESS_PERSONAL_RENT_EXPENSES",
        "Allowed personal-property rental expenses after passive loss limits, Schedule 1 line 24b. Suspended expenses remain identified by activity; personal-property losses receive no rental real-estate special allowance.",
    ),
    (
        "EXPLAIN_NONBUSINESS_PERSONAL_RENT_EXPENSES_PAID",
        "Eligible paid rental expenses before passive loss limits. Only the allowed amount is deducted on Schedule 1 line 24b.",
    ),
    (
        "EXPLAIN_NONBUSINESS_ROYALTY_INCOME",
        "Schedule E nonbusiness royalties less eligible paid expenses, included through Schedule 1 line 5. Portfolio income for investment-tax and EIC rules; not SE earnings, IRA compensation or QBI. Nonbusiness losses cannot create an NOL.",
    ),
    (
        "EXPLAIN_NONQUALIFIED_ANNUITY_INVESTMENT_INCOME",
        "Taxable nonqualified annuity income enters Form 8960 line 3. Qualified retirement-plan distributions remain outside net investment income.",
    ),
    (
        "EXPLAIN_NUA_SALE_N_N",
        "Deferred NUA realized on sale is long-term gain and excluded from NIIT. Remaining gain or loss follows the post-distribution holding period, or is long-term for inherited shares; original pension income is not repeated.",
    ),
    ("EXPLAIN_N_10", "Penalty on early withdrawal of savings."),
    ("EXPLAIN_N_11", "IRA deduction."),
    ("EXPLAIN_N_12", "Student loan interest deduction."),
    ("EXPLAIN_N_13", "Total adjustments to income, Form 1040 line 10."),
    ("EXPLAIN_N_14", "Taxable interest, Form 1040 line 2b."),
    ("EXPLAIN_N_15", "Ordinary dividends, Form 1040 line 3b."),
    ("EXPLAIN_N_16", "Taxable nonqualified annuities; qualified plans and IRAs are excluded."),
    (
        "EXPLAIN_N_17",
        "Nonbusiness personal-property rent, hobby sales income and a fiduciary's box 14 code H adjustment.",
    ),
    ("EXPLAIN_N_18", "Total investment income: lines 1, 2, 3, 4c, 5d, 6 and 7."),
    (
        "EXPLAIN_N_19",
        "Penalty on early withdrawal of savings and the estate-tax deduction for inherited investment income.",
    ),
    ("EXPLAIN_N_2", "Rental real estate, royalties, partnerships and S corporations (Schedule E)."),
    ("EXPLAIN_N_20", "Lines 9d and 10."),
    ("EXPLAIN_N_21", "Line 8 less line 11, not below zero."),
    (
        "EXPLAIN_N_22",
        "Adjusted gross income plus the foreign earned-income exclusion and a fiduciary's modified-AGI adjustment.",
    ),
    (
        "EXPLAIN_N_23",
        "$250,000 joint or surviving spouse, $125,000 married filing separately, $200,000 otherwise.",
    ),
    ("EXPLAIN_N_24", "Line 13 less line 14, not below zero."),
    ("EXPLAIN_N_25", "The smaller of line 12 or line 15."),
    ("EXPLAIN_N_26", "3.8 percent of line 16."),
    (
        "EXPLAIN_N_27",
        "Net section 1231 gain or loss from Part I lines 2 through 7 of every source on this return.",
    ),
    (
        "EXPLAIN_N_28",
        "Your nonrecaptured net section 1231 losses from the five preceding years (2021 through 2025), the whole balance. The smaller of this and line 8 is ordinary income on line 13.",
    ),
    ("LINE_PARTNERSHIP_N_SECTION1231", "Partnership {0} section 1231"),
    ("LINE_AMT_PARTNERSHIP_N_SECTION1231", "AMT partnership {0} section 1231"),
    ("LINE_AMT_SCORP_N_SECTION1231", "AMT S corporation {0} section 1231"),
    ("LINE_FORM4797_LINE12", "Form 4797 ordinary section 1231 loss"),
    ("LINE_FORM4797_LINE13", "Form 4797 section 1231 lookback recapture"),
    (
        "EXPLAIN_FORM4797_LINE12",
        "The net section 1231 loss from Part I line 8 is an ordinary loss on Part II line 12, before return-level loss limits.",
    ),
    (
        "EXPLAIN_FORM4797_LINE13",
        "The section 1231 gain reclassified as ordinary income by the five-year lookback, from Part I line 8 and line 9 to Part II line 13.",
    ),
    (
        "EXPLAIN_PARTNERSHIP_N_SECTION1231",
        "Schedule K-1 (Form 1065) box 10 business-property net section 1231 gain or loss after the basis, at-risk and passive limits: Part I line 2, netted on line 8.",
    ),
    (
        "ISSUE_K1_PTP_PROPERTY",
        "Business-property gains, losses and their carryovers in a publicly traded partnership need its separate property and passive-loss worksheets; they are not calculated.",
    ),
    (
        "EXPLAIN_SCORP_N_SECTION1231",
        "Schedule K-1 (Form 1120-S) box 9 net section 1231 gain or loss after the basis, at-risk and passive limits: Part I line 2, netted on line 8.",
    ),
    (
        "EXPLAIN_INSTALLMENT_N_SECTION1231",
        "The section 1231 gain of an earlier-year installment sale of business or rental property, from Form 6252 line 26: Part I line 4, netted on line 8.",
    ),
    ("EXPLAIN_N_29", "Line 8 less line 9: long-term capital gain on Schedule D line 11."),
    ("EXPLAIN_N_3", "Farm income or loss (Schedule F)."),
    ("EXPLAIN_N_4", "Total additional income, Form 1040 line 8."),
    ("EXPLAIN_N_5", "Educator expenses."),
    ("EXPLAIN_N_6", "Health savings account deduction (Form 8889)."),
    ("EXPLAIN_N_7", "Deductible part of self-employment tax (Schedule SE line 13)."),
    ("EXPLAIN_N_8", "Self-employed SEP, SIMPLE and qualified plans."),
    ("EXPLAIN_N_9", "Self-employed health insurance deduction."),
    (
        "EXPLAIN_N_AMOUNT_REALIZED",
        "Amount realized on the foreclosure, repossession or deed in lieu: the smaller of the property's fair market value and the debt for a recourse loan, or the whole debt for a nonrecourse loan.",
    ),
    (
        "EXPLAIN_N_BUILDING",
        "Loss on the building and improvements: the building part of the sale price less its share of selling expenses and its adjusted basis, a section 1231 loss (Part I line 2).",
    ),
    (
        "EXPLAIN_N_CARRY_REMAINING",
        "Retained prior-year balance after 2026 use and expiration; excludes newly generated 2026 excess taxes and their carryback.",
    ),
    (
        "EXPLAIN_N_CARRY_USED",
        "Prior balance used after current-year taxes, oldest original year first within this category and tax system.",
    ),
    (
        "EXPLAIN_N_CURRENT_LOSS_HISTORY",
        "Current deductible net section 1231 loss entering the five-year lookback history. This is not an additional deduction or an NOL carryforward.",
    ),
    ("EXPLAIN_N_EARLIER_PAYMENTS", "Payments received before 2026."),
    (
        "EXPLAIN_N_EXPENSE_N_DE_MINIMIS",
        "Rental cost expensed under the annual de minimis election; no annual dollar cap or QBI depreciable-property basis. Attach a statement titled Section 1.263(a)-1(f) de minimis safe harbor election with taxpayer name, address, TIN and election declaration to the timely original return, including extensions. This on-screen calculation does not file that statement.",
    ),
    (
        "EXPLAIN_N_EXPENSE_N_DE_MINIMIS_2",
        "Business cost expensed under the annual de minimis election; no annual dollar cap or QBI depreciable-property basis. Attach a statement titled Section 1.263(a)-1(f) de minimis safe harbor election with taxpayer name, address, TIN and election declaration to the timely original return, including extensions. This on-screen calculation does not file that statement.",
    ),
    (
        "EXPLAIN_N_GAIN",
        "Line 24 less line 25: a section 1231 gain on Form 4797 line 4, netted with the return's other business-property gains and losses.",
    ),
    ("EXPLAIN_N_GAIN_2", "Line 24 less line 25: a capital gain on Schedule D."),
    (
        "EXPLAIN_N_INCOME",
        "Installment sale income: line 22 times the gross profit percentage (line 16 divided by line 18).",
    ),
    (
        "EXPLAIN_N_INSTALLMENT_GAIN",
        "Section 1231 gain from this installment sale (Form 6252 line 26); the rest of the gain is taxed as later payments arrive.",
    ),
    (
        "EXPLAIN_N_IRA_REPAYMENT_N_INCOME_REDUCTION",
        "Repayment reduces IRA taxable income, limited to the qualified taxable pool. It does not reduce original gross distributions or withholding.",
    ),
    (
        "EXPLAIN_N_IRA_REPAYMENT_N_LATER_DEPOSITS",
        "Actual later-year repayments included in this original-year calculation. If the original return was already filed, an amended return may be required.",
    ),
    (
        "EXPLAIN_N_IRA_REPAYMENT_N_QUALIFIED_GROSS",
        "Line 15c worksheet qualified distribution pool includes all four applicable types for this owner, even when only one was repaid.",
    ),
    (
        "EXPLAIN_N_IRA_REPAYMENT_N_QUALIFIED_TAXABLE",
        "Taxable part of the owner's qualified distribution pool before repayment, using the Form 8606 ordinary-distribution ratio.",
    ),
    (
        "EXPLAIN_N_IRA_REPAYMENT_N_TAXABLE_AFTER",
        "Form 8606 line 15c ordinary IRA taxable amount after repayment. Roth conversions remain in Part II and are added separately to the return.",
    ),
    (
        "EXPLAIN_N_IRA_REPAYMENT_N_YEAR_END",
        "Form 8606 line 6 value after removing repayments made in 2026 of these 2026 distributions. Later-year deposits do not change this value.",
    ),
    (
        "EXPLAIN_N_LAND",
        "Land part of the sale price less its share of selling expenses and the land cost: a section 1231 gain or loss (Part I line 2).",
    ),
    (
        "EXPLAIN_N_NONDEDUCTIBLE_LOSS",
        "Loss on a main home the lender took: not deductible and not reported.",
    ),
    ("EXPLAIN_N_N_1A", "Net farm profit or loss from Schedule F line 34."),
    (
        "EXPLAIN_N_N_1B",
        "Conservation Reserve Program payments included on Schedule F and excluded from self-employment earnings.",
    ),
    ("EXPLAIN_N_N_4A", "Line 3 times 92.35% when more than zero."),
    (
        "EXPLAIN_N_N_4B",
        "The total of lines 15 and 17: the farm and nonfarm optional method amounts.",
    ),
    (
        "EXPLAIN_N_N_4C",
        "Net earnings from self-employment: line 4a, or the optional methods, and zero under $400.",
    ),
    (
        "EXPLAIN_N_N_FORM_N_BOX0_N_DEPOSIT",
        "2027 regular deposit designated for 2026: the IRA custodian includes this principal on its 2026 Form 5498. This is part of the entered 2026 contribution, not an additional 2027 contribution or a Form 1099-R distribution.",
    ),
    (
        "EXPLAIN_ROTH_FORM_PART_III",
        "This person's Form 8606 Part III uses actual nonqualified distributions and the annual Roth ordering balances. Qualified payments and rollovers are excluded. First-home expenses, qualified disaster amounts and accepted repayments follow their existing calculations; excess repayments remain separately reported basis for next year. Amounts retain cents. Stop lines follow the form's instructions.",
    ),
    (
        "EXPLAIN_INHERITED_IRA_FORM",
        "Separate Form 8606 for this beneficiary and deceased owner, using the inherited group's existing opening basis, distributions, qualified exclusions, recovery and carry. The displayed ratio does not replace the full fraction used for dollars.",
    ),
    (
        "EXPLAIN_INHERITED_ROTH_FORM",
        "Separate Form 8606 for this inherited Roth group, using the deceased owner's five-year period and existing contribution/conversion basis after qualified HSA funding. Other deceased owners and personal IRA basis remain separate.",
    ),
    (
        "EXPLAIN_IRA_FORM_PART_I",
        "This person's actual traditional IRA basis worksheet. Qualified reservist repayments are current nondeductible basis on line 1, outside the regular contribution limit. The displayed fraction has four decimal places; monetary amounts retain the calculation's full fraction and separate whole-dollar rounding. No distribution or conversion stops after lines 1–3 and 14.",
    ),
    (
        "EXPLAIN_IRA_FORM_RETAINED_WORKSHEET",
        "Pub. 590-B Worksheet 1-1 determines the deduction before final IRA taxable income. Its retained exclusion goes to Form 8606 line 13; lines 6–12 are skipped rather than replaced by a different ratio. Conversion and ordinary taxable portions keep the existing worksheet's separate allocation.",
    ),
    (
        "EXPLAIN_IRA_FORM_PART_II",
        "This person's traditional-to-Roth IRA conversion. Part I supplies its recovered basis when applicable; otherwise line 17 shows the basis available for the conversion. A line 18 at or below zero adds no taxable income and creates no deductible loss. The existing annual tax and basis calculation is unchanged.",
    ),
    (
        "EXPLAIN_IRA_FORM_RELIEF",
        "The existing qualified disaster owner supplies line 15b, before spreading that amount on Form 8915-F. Line 15c removes that amount and the existing accepted special-repayment reduction from line 15a. These relief amounts retain source cents; later-year repayment basis remains separately reported.",
    ),
    (
        "EXPLAIN_N_N_IRA_BASIS_CARRYFORWARD",
        "Unused traditional IRA basis retained for a later distribution; each spouse has a separate basis.",
    ),
    (
        "EXPLAIN_N_N_IRA_BASIS_RECOVERED",
        "Form 8606 tax-free recovery of nondeductible contributions across all this filer's traditional IRAs.",
    ),
    (
        "EXPLAIN_N_N_IRA_BASIS_TAXABLE",
        "Taxable traditional IRA distributions after owner-level pro-rata basis allocation, before any separately reported special repayment reduction.",
    ),
    (
        "EXPLAIN_N_N_IRA_CURRENT_NONDEDUCTIBLE_BASIS",
        "Form 8606 line 1: regular nondeductible contributions plus qualified reservist repayments. Repayments do not use annual contribution limits or create a deduction.",
    ),
    (
        "EXPLAIN_N_N_IRA_INCOMING_PLAN_BASIS",
        "After-tax employer-plan contributions rolled into this owner's traditional IRA by year-end. Added once to basis; not a deductible or annual-limit contribution.",
    ),
    (
        "EXPLAIN_N_N_IRA_OUTSTANDING_ROLLOVER",
        "Completed next-year IRA-to-IRA rollovers included in the basis worksheet's year-end value, without treating them as taxable withdrawals.",
    ),
    (
        "EXPLAIN_N_N_IRA_RESERVIST_BASIS",
        "Current-year qualified reservist repayments added as nondeductible IRA basis, outside regular contribution limits and without reversing original distribution income.",
    ),
    (
        "EXPLAIN_N_N_N",
        "Retained Form 4952 election applied under Schedule D Tax Worksheet lines 3–9 before foreign capital-gain-excess and special-rate caps. Current deductions and AGI are unchanged.",
    ),
    (
        "EXPLAIN_N_N_NEXT_LOOKBACK",
        "Nonrecaptured loss-year balance remaining for next year's section 1231 lookback, after current recapture and expiry.",
    ),
    (
        "EXPLAIN_N_N_N_2",
        "Base-year Foreign Earned Income Tax Worksheet. Excess gains reduce net capital gain before dividends; special-rate pools are recomputed before the net-gain cap. Skip tax computation when allocated taxable income is nonpositive.",
    ),
    (
        "EXPLAIN_N_N_N_3",
        "Net profit or loss from Schedule C and partnership self-employment earnings (K-1 box 14 code A).",
    ),
    ("EXPLAIN_N_N_N_4", "Combined net farm and nonfarm profit or loss."),
    (
        "EXPLAIN_N_N_N_5",
        "Self-employment tax: 12.4% Social Security up to the wage base left after wages, plus 2.9% Medicare.",
    ),
    ("EXPLAIN_N_N_N_6", "Deduction for one-half of self-employment tax, Schedule 1 line 15."),
    (
        "EXPLAIN_N_N_N_SECTION179",
        "K-1 section 179 allowed on Form 4562 line 12 after your dollar and business-income limits: Schedule E line 28 column (j) of this nonpassive activity, deducted from its box 1 income; it reduces QBI and, for a general partner, box 14 code A earnings.",
    ),
    (
        "EXPLAIN_N_N_N_SECTION179_CARRYFORWARD",
        "K-1 section 179 disallowed by the business income limit, carried to your 2027 Form 4562 line 10 (Form 4562 line 13). Your basis in the interest already fell by the full K-1 amount this year; on a disposition, basis rises by the unused carryover immediately before the transfer (Regulations section 1.179-3(h)).",
    ),
    (
        "EXPLAIN_N_N_N_SECTION179_PRIOR_CARRYOVER",
        "K-1 section 179 carried from your 2025 Form 4562 line 13, entering Form 4562 line 10 after this year's elections. It already reduced your basis in the year the entity allocated it.",
    ),
    (
        "EXPLAIN_N_N_RECAPTURE",
        "Section 1250 recapture: the additional depreciation, the depreciation allowed over straight line, is ordinary income. Straight-line nonresidential real property placed in service after 1986 has none except the section 50(c) reduction of an investment credit; the gain up to the rest of the depreciation is unrecaptured section 1250 gain.",
    ),
    (
        "EXPLAIN_N_N_RECAPTURE_2",
        "Section 1245 recapture: the smaller of the gain and the depreciation on line 24, ordinary income through line 33 to line 14.",
    ),
    (
        "EXPLAIN_N_N_ROTH_FORM_N_BOX10",
        "The receiving Roth IRA custodians report their respective qualified 529 transfers in 2026 Form 5498 box 10, rather than box 2. This is the combined amount across all receiving accounts. It shares annual IRA contribution room but is not an ordinary Roth contribution, current income, or a Saver's Credit contribution.",
    ),
    (
        "EXPLAIN_N_N_ROTH_SOURCE_CONTRIBUTION_BASIS",
        "The combined source 529 contribution portions retain contribution character in the beneficiary's Roth IRA. Reconcile each transfer with its 529 distributor's Form 1099-Q box 3; source earnings remain Roth earnings.",
    ),
    (
        "EXPLAIN_N_N_ROTH_TRANSFER_N_AMOUNT",
        "Direct 2026 transfer {0,number,0} from 529 account reference {1} on {2,date}. Reconcile this transfer's contribution and earnings portions with its Form 1099-Q and the administrator's five-year seasoned available balance; do not reuse another account's balance.",
    ),
    (
        "EXPLAIN_N_N_TIMELY_RETURN_EARNINGS",
        "Form 1040 line 4b: sum of positive attributable earnings from each corrected account, taxable on the 2026 contribution-year return including a distribution received in 2027. A custodian-allocated loss cannot offset another account's earnings or create a separate loss deduction. A timely section 408(d)(4) return incurs neither the 6% excess-contribution tax nor the 10% early-distribution tax (Form 5329 exception 21).",
    ),
    (
        "EXPLAIN_N_N_TIMELY_RETURN_GROSS",
        "Form 1040 line 4a for the contribution-year return: sum of the actual cash distributed in the timely distinct-account corrections, each equal to its original principal plus attributable net income or minus a net loss. Original principal is not a retained regular contribution, deduction, IRA basis addition, Saver's Credit contribution, or taxable income; do not report a corrective payment again as an ordinary IRA withdrawal.",
    ),
    (
        "EXPLAIN_N_ORDINARY",
        "Section 1252 recapture taken from line 24 first, ordinary income on Form 4797 line 16.",
    ),
    (
        "EXPLAIN_N_ORDINARY_2",
        "Loss from abandoning or scrapping business property: its adjusted basis, an ordinary loss (Part II line 11).",
    ),
    (
        "EXPLAIN_N_ORDINARY_3",
        "Gain or loss on business property held too short a time, ordinary (Part II line 11): one year or less, or for livestock kept for draft, breeding, dairy or sport less than 24 months (cattle and horses) or 12 months (other livestock), Part II line 18.",
    ),
    ("EXPLAIN_N_PAYMENTS", "Principal received in 2026, without interest."),
    (
        "EXPLAIN_N_SECTION1231",
        "Loss on depreciable business property held long enough: a section 1231 loss (Part I line 2, or line 7 for livestock), sale price less adjusted basis.",
    ),
    (
        "EXPLAIN_N_TAXABLE",
        "The 2026 Schedule 1 line 8z amount is zero until the main home becomes usable again. Otherwise it is cumulative insurer payments above the whole temporary living-expense increase, less taxable payments already reported before 2026. Earlier payments are included when use is regained in 2026; later payments are included in their receipt year. A federally declared disaster-area payment is excluded. Property-damage reimbursement is separate.",
    ),
    (
        "EXPLAIN_N_TEMPORARY_INCREASE",
        "Temporary increase in reasonable living expenses while the main home could not be used: actual expenses less normal expenses for the same period, but not below zero. If use has not yet been regained, this is the amount recorded through 2026 and remains open for later years.",
    ),
    ("EXPLAIN_N_TOTAL_PAYMENTS", "Line 21; there is no line 20 amount after the year of sale."),
    (
        "EXPLAIN_N_UNRECAPTURED_N",
        "Unrecaptured section 1250 gain of this property: the smaller of its depreciation and its building gain, before the return-wide section 1231 netting.",
    ),
    (
        "EXPLAIN_N_UNRECAPTURED_N_2",
        "Unrecaptured section 1250 gain in line 26: the gain of each payment is unrecaptured gain first, until the sale's total is reported (Unrecaptured Section 1250 Gain Worksheet line 4).",
    ),
    (
        "EXPLAIN_N_WITHHOLDING",
        "Federal income tax actually withheld on insurers' 2026 Forms 1099, claimed on Form 1040 line 25b. Prior-year withholding is not credited again.",
    ),
    (
        "EXPLAIN_OPTIONAL_SE_N_N_FARM",
        "Optional farm earnings: two-thirds of gross farm income, capped at $7,560; no further 92.35% reduction.",
    ),
    (
        "EXPLAIN_OPTIONAL_SE_N_N_NET",
        "Schedule SE line 4c combines regular earnings or loss with elected optional earnings.",
    ),
    (
        "EXPLAIN_OPTIONAL_SE_N_N_NONFARM",
        "Nonfarm optional earnings share the remaining $7,560 cap.",
    ),
    (
        "EXPLAIN_PARTNERSHIP_N_FUEL_N",
        "Partner-share K-1 box 20 code C fuel quantities at the Form 4136 printed rate or the authorized line 13c 24.4-cent-tax write-in rate, using GGE or DGE on the applicable alternative-fuel rows. This is a partner-level payment credit; it is not added again to the K-1 box 1 amount.",
    ),
    (
        "EXPLAIN_PARTNERSHIP_N_N_BACKUP_WITHHOLDING",
        "K-1 box 15 code O backup withholding, included once on Form 1040 line 25c.",
    ),
    (
        "EXPLAIN_PARTNERSHIP_N_N_CASH_CHARITY",
        "K-1 box 13 code A cash contributions (60%), Schedule A line 11 with your own cash gifts; not part of Form 8582.",
    ),
    (
        "EXPLAIN_PARTNERSHIP_N_N_DIVIDENDS",
        "K-1 box 6a ordinary dividends, reported on Form 1040 line 3b and Schedule B with the partnership's name; box 6b qualified dividends are included and also on line 3a.",
    ),
    (
        "EXPLAIN_PARTNERSHIP_N_N_EXEMPT_INTEREST",
        "K-1 box 18 code A tax-exempt interest, Form 1040 line 2a; it increases outside basis.",
    ),
    (
        "EXPLAIN_PARTNERSHIP_N_N_INTEREST",
        "K-1 box 5 interest, reported on Form 1040 line 2b and Schedule B with the partnership's name.",
    ),
    (
        "EXPLAIN_PARTNERSHIP_N_N_LONG_GAIN",
        "K-1 box 9a net long-term capital gain or loss, Schedule D line 12, including the box 9b and 9c parts.",
    ),
    (
        "EXPLAIN_PARTNERSHIP_N_N_SHORT_GAIN",
        "K-1 box 8 net short-term capital gain or loss, Schedule D line 5.",
    ),
    (
        "EXPLAIN_PARTNERSHIP_N_PRIOR_FUEL_CREDIT_RECOVERY",
        "Partner fuel credit claimed on the 2025 Form 1040 filed in 2026, recognized once as 2026 Schedule E nonpassive income after a full prior tax benefit. It is not current K-1 box 1 or box 14 income and is not a second Form 4136 payment.",
    ),
    (
        "EXPLAIN_PAYMENTS",
        "Income tax withholding, estimated/extension payments and eligible excess payroll withholding.",
    ),
    (
        "EXPLAIN_PENSION_CONTRIBUTION_DEDUCTION",
        "Schedule 1 line 24f employee-funded pension deduction. Code H remains in W-2 wages and is not deducted a second time there. Traditional IRA capacity is reduced by these contributions under Pub. 590-A.",
    ),
    (
        "EXPLAIN_PENSION_CONTRIBUTION_LIMIT_TAXPAYER",
        "Annual section 501(c)(18)(D) limit after the employee's compensation and other regular elective deferrals. No age-based increase applies to this trust contribution limit.",
    ),
    (
        "EXPLAIN_PENSION_N_COMMON_MONTHLY_EXCLUSION",
        "Common monthly exclusion before simultaneous-annuitant allocation; retain with whole-contract recovery records. Do not allocate the recipient's already reduced monthly exclusion again.",
    ),
    (
        "EXPLAIN_PENSION_N_EARLIER_COMMON_RECOVERY",
        "Common cost recovered in earlier payment months of this year by other annuitants; this is not an additional exclusion on this recipient's return.",
    ),
    (
        "EXPLAIN_PENSION_N_NEXT_YEAR_IRA_BASIS",
        "After-tax employer-plan cash deposited into the IRA in 2027. Add once to the 2027 IRA basis record; this is neither 2026 IRA basis nor an outstanding IRA-to-IRA rollover.",
    ),
    (
        "EXPLAIN_PENSION_N_N_BASIS_RECOVERY",
        "Simplified Method exclusion using original monthly recovery and remaining cost; pre-1987 starts have no lifetime cost cap.",
    ),
    (
        "EXPLAIN_PENSION_N_N_BASIS_RECOVERY_3",
        "Distribution multiplied by unrecovered investment divided by the same nonforfeitable account or separate-contract balance, limited to the distribution.",
    ),
    (
        "EXPLAIN_PENSION_N_N_INCOME_EXCLUSION",
        "Tax-free pension income, limited to the contributory portion for railroad benefits; vested dual and supplemental benefits remain taxable.",
    ),
    (
        "EXPLAIN_PENSION_N_N_MONTHLY_EXCLUSION",
        "Recipient monthly exclusion at the final entered payment month. Retain the common worksheet and allocation history; entitlement changes can alter the recipient share, while cost-of-living changes alone do not.",
    ),
    (
        "EXPLAIN_PENSION_N_N_REMAINING_BASIS",
        "Unrecovered investment after this distribution, before later contributions or adjustments.",
    ),
    (
        "EXPLAIN_PENSION_N_N_ROLLED_BASIS_2",
        "After-tax investment transferred to the receiving retirement account. Retain its basis record; this is not a deductible annual contribution.",
    ),
    (
        "EXPLAIN_PENSION_N_REMAINING_BASIS",
        "Unrecovered whole-contract cost after the entered payment months. Reconcile any later recovery by other annuitants before carrying the common record to next year's worksheet.",
    ),
    (
        "EXPLAIN_PERSONAL_CASUALTY_DEDUCTION",
        "Ordinary 2026 personal-use disaster loss requires itemizing; qualified federal major-disaster loss is also added to the standard deduction.",
    ),
    (
        "EXPLAIN_PERSONAL_DEBT_CANCELLATION",
        "Taxable personal debt cancellation on Schedule 1 line 8c. Increases AGI and nonbusiness unearned income; does not create IRA/EIC compensation, self-employment tax or net investment income.",
    ),
    (
        "EXPLAIN_PERSONAL_REFUELING_CREDIT",
        "Form 8911 line 10 and Schedule 3 line 6j: personal-use credit after the per-unit limit and tax above tentative minimum tax.",
    ),
    (
        "EXPLAIN_PERSONAL_REFUELING_CREDIT_UNUSABLE",
        "Personal-use refueling credit disallowed by the tax limit expires; it does not carry to another year.",
    ),
    (
        "EXPLAIN_PERSONAL_RENTAL_N_ASSET_N_N",
        "Purchased rental equipment: bonus and GDS depreciation before passive limits; remaining basis is reduced even when the deduction is suspended. Section 179 is unavailable. AMT deductions use the separate retained schedule when required; the passive worksheet determines the allowed income adjustment.",
    ),
    (
        "EXPLAIN_PERSONAL_RENT_N_INVESTMENT_INCOME",
        "Net rental income after prior suspended expenses, recharacterized for Form 4952 when less than 30% of the rental activity's unadjusted basis is depreciable. Ordinary passive rental income stays outside this investment-interest pool; Form 8960 has a separate definition.",
    ),
    (
        "EXPLAIN_PERSONAL_SALE_DISALLOWED_LOSS",
        "Personal-use losses are disallowed item by item before capital-gain netting; they cannot offset other gains or create carryforwards. Proceeds reported on a 1099-K are not automatically taxable income.",
    ),
    (
        "EXPLAIN_PREMIUM_CREDIT_ADVANCE",
        "Advance payments from all entered monthly policy statements, after any Part IV share of a shared policy.",
    ),
    (
        "EXPLAIN_PREMIUM_CREDIT_REFUNDABLE",
        "Additional premium tax credit after subtracting advance payments.",
    ),
    (
        "EXPLAIN_PREMIUM_CREDIT_REPAYMENT",
        "2026 requires full repayment of excess advances; the earlier repayment caps no longer apply.",
    ),
    (
        "EXPLAIN_PREMIUM_TAX_CREDIT",
        "Married filing separately without the domestic abuse or abandonment exception: no credit is allowed and the advance credit is repaid in full (Form 8962 lines 11 to 23 column (f)).",
    ),
    (
        "EXPLAIN_PREMIUM_TAX_CREDIT_2",
        "Household income below the poverty line: allowed because advance credit was paid on the Marketplace's enrollment estimate, at the 2.10% applicable percentage.",
    ),
    (
        "EXPLAIN_PREMIUM_TAX_CREDIT_3",
        "Allowed credit before advances, using the 2026 100%–400% poverty range and any Part IV shares of shared policies.",
    ),
    (
        "EXPLAIN_PREVIOUSLY_OWNED_CLEAN_VEHICLE_CREDIT",
        "Form 8936 line 18 and Schedule 3 line 6m: used clean-vehicle credit after earlier personal credits; unused personal credit expires.",
    ),
    (
        "EXPLAIN_PREVIOUSLY_OWNED_CLEAN_VEHICLE_CREDIT_UNUSABLE",
        "Previously owned personal-use credit lost to the Form 1040 tax limit; it does not carry forward.",
    ),
    (
        "EXPLAIN_PREVIOUSLY_OWNED_CLEAN_VEHICLE_TRANSFER_REPAYMENT",
        "2026 Schedule 2 line 1c: repay transferred used-vehicle credit when both MAGI years exceed their limits or the vehicle was resold within 30 days.",
    ),
    (
        "EXPLAIN_PRIOR_ATNOL_DEDUCTION",
        "Ordinary prior ATNOL is limited to 90% of AMTI before this deduction. Documented special losses can absorb the remaining AMTI. This does not change regular AGI, self-employment tax or QBI.",
    ),
    (
        "EXPLAIN_PRIOR_INCOME_REPAYMENT_REVIEW_TOTAL",
        "Combined earlier-year unemployment and railroad principal and net negative Social Security benefits, before tax-line rounding. This is a review threshold, not a claim that every dollar was taxable in its original year. Unentered repayments of other income must be reported in Additional coverage checks.",
    ),
    (
        "EXPLAIN_PRIOR_NOL_DEDUCTION",
        "Pre-2018 loss plus post-2017 loss limited to 80% of taxable income before NOL and QBI, less the pre-2018 loss. Reported as negative Schedule 1 income, not a reduction to SE earnings.",
    ),
    (
        "EXPLAIN_PRIOR_YEAR_CARE_CREDIT",
        "2025 care first paid in 2026, using the prior year expense/earnings limits and 20%-35% rate. Added to current care before the current tax-liability limit; 2026 instructions remain draft.",
    ),
    (
        "EXPLAIN_PROPERTY_ROLLOVER_N_N",
        "Retained sale proceeds allocate between distribution value and post-distribution gain or loss. The rolled portion recognizes neither sale gain nor sale loss; no duplicate capital-sale entry is needed.",
    ),
    (
        "EXPLAIN_PUBLIC_SAFETY_MEDICAL_BEFORE_COORDINATION",
        "Unexcluded premiums under this election before business long-term-care coordination. The shared per-person long-term-care medical lines replace the LTC portion of this subtotal; do not add both. Ordinary medical premiums enter itemization once.",
    ),
    (
        "EXPLAIN_PUBLIC_SAFETY_N_EXCLUDED",
        "Section 402(l) election: excluded after pension basis recovery, limited to otherwise taxable distributions and the officer's remaining $3,000 annual allowance. Preserve the PSO reporting notation; no duplicate medical deduction.",
    ),
    (
        "EXPLAIN_QBI_ACTIVE_MINIMUM",
        "The 2026 minimum deduction: $400 when active qualified business income is at least $1,000.",
    ),
    ("EXPLAIN_QBI_ACTIVITY", "Qualified business income after netting losses."),
    ("EXPLAIN_QBI_ACTIVITY_2", "Qualified business income or loss of this trade or business."),
    (
        "EXPLAIN_QBI_ACTIVITY_COMPONENT",
        "Qualified business income component after the W-2 wage and UBIA limit and the phase-in reduction.",
    ),
    (
        "EXPLAIN_QBI_ACTIVITY_LIMITED",
        "The greater of the 20% amount reduced by the phase-in and the wage and UBIA limit, before the patron reduction.",
    ),
    (
        "EXPLAIN_QBI_ACTIVITY_PATRON_REDUCTION",
        "The patron reduction allocable to this trade or business (Schedule D line 6).",
    ),
    (
        "EXPLAIN_QBI_PATRON_REDUCTION",
        "The Schedule D patron reduction for Form 1099-PATR box 7 qualified payments: the lesser of 9% of the QBI allocable to them or 50% of the allocable W-2 wages, which is zero without farm W-2 wages.",
    ),
    ("EXPLAIN_QBI_ACTIVITY_TWENTY_PERCENT", "20% of line 2."),
    (
        "EXPLAIN_QBI_AFTER_COOPERATIVE",
        "Line 37 less the cooperative deduction, which this return does not have.",
    ),
    (
        "EXPLAIN_QBI_BEFORE_INCOME_LIMIT",
        "Qualified business income deduction before the income limitation.",
    ),
    ("EXPLAIN_QBI_COMPONENT", "Qualified business income component."),
    (
        "EXPLAIN_QBI_DEDUCTION",
        "QBI after service-business applicability, loss netting, phased wage/property limits and taxable-income cap, with the 2026 active-business minimum.",
    ),
    ("EXPLAIN_QBI_INCOME_LESS_GAIN", "Taxable income less net capital gain, not less than zero."),
    ("EXPLAIN_QBI_INCOME_LIMIT", "Income limitation: 20% of line 13."),
    ("EXPLAIN_QBI_LIMITED_DEDUCTION", "The smaller of line 10 and line 14."),
    (
        "EXPLAIN_QBI_LOSS_CARRYFORWARD",
        "Net qualified business loss including the prior QBI carryforward, after service-business applicability. Carries to the next year without a second AGI deduction.",
    ),
    (
        "EXPLAIN_QBI_NET",
        "Total qualified business income after the prior-year loss, not less than zero.",
    ),
    (
        "EXPLAIN_QBI_NET_CAPITAL_GAIN",
        "Net capital gain: qualified dividends plus the net capital gain.",
    ),
    ("EXPLAIN_QBI_PRIOR_LOSS", "Qualified business net loss carried forward from the prior year."),
    (
        "EXPLAIN_QBI_PRIOR_REIT_PTP_LOSS",
        "Qualified REIT dividends and PTP loss carried forward from the prior year.",
    ),
    (
        "EXPLAIN_QBI_REIT_PTP",
        "Qualified REIT dividends and publicly traded partnership income or loss.",
    ),
    ("EXPLAIN_QBI_REIT_PTP_COMPONENT", "REIT and PTP component: 20% of line 8."),
    (
        "EXPLAIN_QBI_REIT_PTP_NET",
        "Total qualified REIT dividends and PTP income, not less than zero.",
    ),
    (
        "EXPLAIN_QBI_TAXABLE_INCOME",
        "Taxable income before the qualified business income deduction.",
    ),
    ("EXPLAIN_QBI_TOTAL", "Total qualified business income or loss."),
    (
        "EXPLAIN_QCD_ADJUSTMENT_CARRYFORWARD",
        "Total unused deductible-contribution adjustment; retain the separately reported owner balances.",
    ),
    (
        "EXPLAIN_QCD_CASH_GIFTS",
        "Nonexcluded qualifying cash gifts enter the ordinary charity deduction calculation once.",
    ),
    (
        "EXPLAIN_QCD_EXCLUSION",
        "Qualified charitable distributions excluded from IRA income, not deducted again as gifts.",
    ),
    (
        "EXPLAIN_QCD_TAXPAYER_ADJUSTMENT_CARRY",
        "Unused deductible-contribution adjustment retained for this owner's next QCD worksheet.",
    ),
    (
        "EXPLAIN_QCD_TAXPAYER_EXCLUSION",
        "2026 exclusion after the $111,000 owner limit and unused prior plus current post-age-70½ IRA deductions. Gross IRA distributions remain reported.",
    ),
    (
        "EXPLAIN_QSBS_N_N_AMT_PREFERENCE",
        "Seven percent of the excluded gain from shares acquired before September 28, 2010; Form 6251 line 2h, before whole-dollar rounding.",
    ),
    (
        "EXPLAIN_QSBS_N_N_EXCLUDED_GAIN",
        "Applicable 50%, 60%, 75% or 100% exclusion of eligible gain for directly purchased original-issue shares held more than five years; Form 8949 code Q.",
    ),
    (
        "EXPLAIN_QSBS_N_N_EXCLUSION_LIMIT",
        "Greater of remaining issuer dollar cap or ten times the original-issue basis of this year's disposed shares.",
    ),
    (
        "EXPLAIN_QSBS_N_N_RATE_N_GAIN",
        "Unexcluded eligible section 1202 gain before Schedule D loss coordination; 100%-excludable shares contribute zero, even if issuer-cap excess remains taxable.",
    ),
    (
        "EXPLAIN_QSBS_N_N_REALIZED_GAIN",
        "Cash proceeds less original-issue basis, before the section 1202 exclusion.",
    ),
    (
        "EXPLAIN_QSBS_N_N_TAXABLE_GAIN",
        "Eligible gain not excluded plus gain above the section 1202(b) issuer limit. Only the former belongs in the 28%-rate pool; the latter has ordinary long-term capital-gain rates.",
    ),
    (
        "EXPLAIN_QSBS_N_N_ZONE_GAIN_THROUGH_N",
        "Source-valued pre-2019 empowerment-zone gain receiving ten additional percentage points of exclusion above its 50% base.",
    ),
    (
        "EXPLAIN_QUALIFIED_NET_DISASTER_LOSS",
        "Provisional 2026 Form 4684 line 15 and Schedule A line 17b: qualified major-disaster losses after a $500 reduction per event and any remaining casualty gains. Available whether itemizing or taking the increased standard deduction.",
    ),
    (
        "EXPLAIN_RAILROAD_PRIOR_YEAR_REPAYMENTS",
        "Fully taxable earlier-year railroad benefits repaid in 2026. Do not net against current pension receipts. The combined return-wide threshold and original-year comparison determine any deduction or credit.",
    ),
    (
        "EXPLAIN_RATE_N_GAIN",
        "Long-term 28%-rate gain from collectibles and unexcluded eligible section 1202 gain after long-term loss carryovers and net short-term losses; capped by net capital gain. Tax is at most 28%, not a flat 28%.",
    ),
    (
        "EXPLAIN_REFUNDABLE_EDUCATION_CREDIT",
        "American opportunity refundable portion using the taxpayer's age/support eligibility.",
    ),
    (
        "EXPLAIN_REGULAR_INCOME_TAX",
        "Regular income tax including an applicable Schedule J election, Form 8615 parent-rate tax, preferential gains and foreign-exclusion stacking, before AMT and credits. Applicable current-year ordinary-tax lookups below $100,000 use the September 2026 draft Tax Table, including Schedule J line 4; its base-year recomputations use the IRS rate schedules.",
    ),
    (
        "EXPLAIN_REGULAR_TOTAL_NOL_CARRYFORWARD",
        "Unexpired prior regular losses plus newly generated current NOL and excess business loss, counted once and reduced by Form 982 after current tax. Retain year and origin; new excess-business-loss components require their own QBI allocation before later use.",
    ),
    (
        "EXPLAIN_REIT_PTP_LOSS_CARRYFORWARD",
        "Remaining prior REIT/PTP loss after qualified section 199A dividends; tracked separately from business QBI and never deducted again from AGI.",
    ),
    (
        "EXPLAIN_RENTAL_N_BUILDING_N_DEPRECIATION",
        "This building's current GDS deduction, mid-month convention; land excluded.",
    ),
    (
        "EXPLAIN_RENTAL_N_DEPRECIATION",
        "39-year nonresidential real property GDS, mid-month convention; land excluded.",
    ),
    (
        "EXPLAIN_RENTAL_N_DEPRECIATION_2",
        "27.5-year residential GDS, mid-month convention; land excluded.",
    ),
    (
        "EXPLAIN_RENTAL_N_EXPENSE_LINE10_LEGAL_AND_PROFESSIONAL",
        "Schedule E line 10: legal and other professional fees.",
    ),
    ("EXPLAIN_RENTAL_N_EXPENSE_LINE11_MANAGEMENT_FEES", "Schedule E line 11: management fees."),
    (
        "EXPLAIN_RENTAL_N_EXPENSE_LINE12_MORTGAGE_INTEREST",
        "Schedule E line 12: mortgage interest paid to banks or other financial institutions.",
    ),
    (
        "EXPLAIN_RENTAL_N_EXPENSE_LINE13A_VEHICLE_LOAN_INTEREST",
        "Schedule E line 13a (2026 draft): vehicle-loan interest.",
    ),
    (
        "EXPLAIN_RENTAL_N_EXPENSE_LINE13B_OTHER_INTEREST",
        "Schedule E line 13b (2026 draft): other interest.",
    ),
    ("EXPLAIN_RENTAL_N_EXPENSE_LINE14_REPAIRS", "Schedule E line 14: repairs."),
    ("EXPLAIN_RENTAL_N_EXPENSE_LINE15_SUPPLIES", "Schedule E line 15: supplies."),
    ("EXPLAIN_RENTAL_N_EXPENSE_LINE16_TAXES", "Schedule E line 16: taxes."),
    ("EXPLAIN_RENTAL_N_EXPENSE_LINE17_UTILITIES", "Schedule E line 17: utilities."),
    (
        "EXPLAIN_RENTAL_N_EXPENSE_LINE19_OTHER",
        "Schedule E line 19: other expenses, with a description, including a section 481(a) decrease.",
    ),
    ("EXPLAIN_RENTAL_N_EXPENSE_LINE5_ADVERTISING", "Schedule E line 5: advertising."),
    ("EXPLAIN_RENTAL_N_EXPENSE_LINE6_AUTO_AND_TRAVEL", "Schedule E line 6: auto and travel."),
    (
        "EXPLAIN_RENTAL_N_EXPENSE_LINE7_CLEANING_AND_MAINTENANCE",
        "Schedule E line 7: cleaning and maintenance.",
    ),
    ("EXPLAIN_RENTAL_N_EXPENSE_LINE8_COMMISSIONS", "Schedule E line 8: commissions."),
    ("EXPLAIN_RENTAL_N_EXPENSE_LINE9_INSURANCE", "Schedule E line 9: insurance."),
    (
        "EXPLAIN_RENTAL_N_LINE22_ALLOWED_LOSS",
        "Rental loss of a property sold in full, including its prior suspended losses: allowed in full because the activity, with its sale gain, has an overall loss (section 469(g)).",
    ),
    (
        "EXPLAIN_RENTAL_N_LINE22_ALLOWED_LOSS_2",
        "Deductible rental real estate loss after the passive activity limits (Form 8582), including a prior-year unallowed loss that the property's own income used.",
    ),
    (
        "EXPLAIN_RENTAL_N_LINE22_NONPASSIVE_LOSS",
        "Rental real estate loss that is not from a passive activity, as for a qualified real estate professional: the loss from line 21, deductible in full.",
    ),
    (
        "EXPLAIN_RENTAL_N_N_N",
        "Rents received, and the 2026 part of a section 481(a) increase reported with them.",
    ),
    ("EXPLAIN_RENTAL_N_N_N_2", "Depreciation expense or depletion (Form 4562)."),
    ("EXPLAIN_RENTAL_N_N_N_3", "Total expenses: lines 5 through 19."),
    ("EXPLAIN_RENTAL_N_N_N_4", "Income or loss: rents less total expenses."),
    (
        "EXPLAIN_REPAYMENT_CREDIT",
        "Section 1341 credit from verified prior-year tax decreases, allowed only by the credit method and not limited to current income tax.",
    ),
    (
        "EXPLAIN_REPAYMENT_CREDIT_POTENTIAL",
        "Verified historical and applicable restoration-year Chapter1 tax decrease; used only by the credit method. The deduction method does not also claim this credit.",
    ),
    (
        "EXPLAIN_REPAYMENT_ITEMIZED_AMOUNT",
        "Repayment included once in the itemized-deduction method before the general 2026 itemized limit. The credit method excludes it entirely.",
    ),
    (
        "EXPLAIN_REPAYMENT_YEAR_N_N",
        "Original-year tax, enhanced senior deduction, elderly credit and earned-income-credit table recomputation after excluding the matching repaid benefits. The Chapter 1 decrease includes AMT, early-distribution tax and changed refundable EIC. NIIT under Chapter 2A is not part of this credit; no user-calculated repayment credit is accepted.",
    ),
    (
        "EXPLAIN_REPAYMENT_YEAR_N_N_EXCLUSION_N",
        "Independent exclusion-only capital worksheet, preserving original and refigured loss utilization.",
    ),
    (
        "EXPLAIN_REQUIRED_DISTRIBUTION_TAX",
        "Form 5329 Part IX tax enters Schedule 2 and total tax without changing income. Later-year corrective payments are income only in their payment year.",
    ),
    (
        "EXPLAIN_RESERVIST_N_N_N_BASIS",
        "Qualified reservist repayments add Form 8606 nondeductible basis. They are neither deductible contributions nor tax-free rollovers of the original income.",
    ),
    (
        "EXPLAIN_RESERVIST_N_N_N_UNREPAID",
        "Original distribution less all entered repayments. A further repayment still needs to meet the two-year deadline; this is not an automatic contribution entitlement.",
    ),
    (
        "EXPLAIN_RESIDENTIAL_ENERGY_CARRYFORWARD_N",
        "2026 Form 5695 line 4: unused residential clean energy credit carried to 2027.",
    ),
    (
        "EXPLAIN_RESIDENTIAL_ENERGY_CREDIT",
        "2026 Form 5695 lines 1–3: the 2025 residential clean energy credit carryforward, limited by tax after preceding credits.",
    ),
    (
        "EXPLAIN_RETIREMENT_N_QUALIFIED_N_GROSS",
        "Gross distribution assigned to this exception, including recovered basis. Shared limits use gross amounts, not just taxable income.",
    ),
    (
        "EXPLAIN_RETIREMENT_N_QUALIFIED_N_TAXABLE",
        "Taxable share qualifying for the early-distribution exception before any special IRA repayment. The separate repayment worksheet may remove ordinary income and the corresponding exception amount.",
    ),
    (
        "EXPLAIN_RETIREMENT_ROLLOVERS",
        "Eligible amounts actually transferred. Gross distributions and withholding remain on the return; only the taxable amount kept enters income.",
    ),
    (
        "EXPLAIN_RMD_N_ACCOUNT_N_MINIMUM",
        "Prior December 31 value plus outstanding incoming rollover, divided by the IRS age-based factor. The joint table applies only to a qualifying sole-beneficiary spouse more than ten years younger. Each account is calculated separately.",
    ),
    (
        "EXPLAIN_RMD_N_CONDITIONAL_REDUCED_SHORTFALL",
        "The 10% estimate is conditional on actually submitting the additional-tax return within the correction window. The App has not filed it; recalculate if the planned filing date or IRS notice/assessment changes.",
    ),
    (
        "EXPLAIN_RMD_N_DEFERRED_TO_N",
        "First 2026 payment elected for April 1, 2027. Excluded from 2026 excess-accumulation tax; this does not postpone the separate December 31, 2027 requirement.",
    ),
    (
        "EXPLAIN_RMD_N_N",
        "Account-group shortfall excludes late corrections and unrelated excess withdrawals. Waiver amounts are requests subject to IRS approval, not granted relief; retain the explanation for Form 5329.",
    ),
    (
        "EXPLAIN_RMD_N_TAX",
        "Separate filer excess-accumulation tax: 10% for fully corrected groups with submitted or explicitly planned filing within the correction window, otherwise 25%; less requested reasonable-cause relief.",
    ),
    (
        "EXPLAIN_ROTH_CONTRIBUTION_MAGI",
        "Modified AGI used only for regular Roth IRA contribution eligibility; taxable conversion income remains on the return.",
    ),
    (
        "EXPLAIN_ROTH_CONVERSION_TAXABLE",
        "Taxable traditional-to-Roth IRA conversions remain in income. Converted amounts are not early withdrawals or regular annual contributions.",
    ),
    (
        "EXPLAIN_ROTH_N_N_CONVERSION_RECAPTURE",
        "Previously taxed conversion principal withdrawn inside its own five-tax-year period without an age or disability exception. This is subject to additional tax, not taxed as income again.",
    ),
    (
        "EXPLAIN_ROTH_N_N_REGULAR_RECOVERED",
        "Regular contribution basis recovered tax-free before conversions and earnings under tax-year aggregation, including eligible current-year contributions and the source contribution portions of qualified 529 rollovers.",
    ),
    (
        "EXPLAIN_ROTH_N_N_TAXABLE",
        "Nonqualified Roth earnings remaining after regular contributions and conversion amounts; qualified distributions are excluded.",
    ),
    (
        "EXPLAIN_ROYALTY_N_LINE4_ROYALTIES",
        "Royalties received from this property before its expenses.",
    ),
    (
        "EXPLAIN_SAVER_AUTOMATIC_WITHDRAWALS_TAXPAYER",
        "Permissible automatic-enrollment withdrawals, including earnings, automatically included once in the Form 8880 distribution lookback. This does not impose early-distribution tax.",
    ),
    (
        "EXPLAIN_SAVER_CREDIT",
        "Retirement and beneficiary-funded ABLE Saver’s Credit sharing a per-person limit, with foreign-income add-back, 2026 income bands and distribution lookback; limited by tax after preceding credits.",
    ),
    (
        "EXPLAIN_SAVINGS_BOND_EXCLUSION",
        "Form 8815 line 14 and Schedule B line 3: line 8 less line 13.",
    ),
    (
        "EXPLAIN_SAVINGS_BOND_EXCLUSION_NOT_ALLOWED",
        "Form 8815 line 14: no exclusion on a married-filing-separately return, or when the bond owner had not turned 24 before the bonds were issued.",
    ),
    (
        "EXPLAIN_SAVINGS_BOND_EXCLUSION_PHASED_OUT",
        "Form 8815 line 14: modified AGI is at or above the top of the 2026 phaseout ($116,800, or $182,650 on a joint return), so none of the interest is excluded.",
    ),
    (
        "EXPLAIN_SAVINGS_WITHDRAWAL_PENALTY",
        "Deductible savings-withdrawal penalty from the payer's statement. Full interest stays on line 2b; the penalty is a separate income adjustment and net-investment-income deduction.",
    ),
    (
        "EXPLAIN_QSBS_AMT_PREFERENCE",
        "7% of the section 1202 exclusion on qualified small business stock acquired before September 28, 2010 (50% and 75% exclusions), an alternative minimum tax preference on Form 6251 line 2h. Stock acquired later has no preference.",
    ),
    (
        "EXPLAIN_SCHEDULE_AI_FULL_YEAR_ITEMIZED",
        "Schedule AI line 4 column (d): the itemized deductions on Form 1040 line 12e, or 0 with the standard deduction.",
    ),
    (
        "EXPLAIN_SCHEDULE_AI_FULL_YEAR_SELF_EMPLOYMENT",
        "Schedule AI Part II column (d): the year's net profit from self-employment, before line 28's 92.35% reduction.",
    ),
    (
        "EXPLAIN_SCHEDULE_AI_FULL_YEAR_SOCIAL_WAGES",
        "Schedule AI Part II column (d): the Social Security wages and tips of the person with self-employment income.",
    ),
    (
        "EXPLAIN_SCHEDULE_B_ORDINARY_DIVIDENDS",
        "Schedule B line 6: ordinary dividends from every payer, carried to Form 1040 line 3b.",
    ),
    (
        "EXPLAIN_SCHEDULE_B_SAVINGS_BOND_EXCLUSION",
        "Schedule B line 3: excludable interest on series EE and I savings bonds, from Form 8815 line 14.",
    ),
    (
        "EXPLAIN_SCHEDULE_B_TAXABLE_INTEREST",
        "Schedule B line 4: interest from every payer less the savings bond exclusion (line 3), carried to Form 1040 line 2b.",
    ),
    (
        "EXPLAIN_SCHEDULE_B_TOTAL_INTEREST",
        "Schedule B line 2: total interest from every payer, before the savings bond exclusion.",
    ),
    (
        "EXPLAIN_SCHEDULE_E_RENTAL_ROYALTY_TOTAL",
        "Total rental real estate and royalty income or loss: each property's income or deductible loss, plus nonbusiness royalties.",
    ),
    (
        "EXPLAIN_SCHEDULE_N_LINE_13Z_FORM_N",
        "Negative Form 8978 adjustment after Schedule 3 line 6l, limited to verified Chapter 1 early-distribution and section 223 HSA additional taxes on 2026 Schedule 2 Part II. Section 4973 excess-contribution excise is excluded. The 2026 draft places other taxes on line 13z; the published 2025 worksheet calls it line 17z.",
    ),
    (
        "EXPLAIN_SCHEDULE_N_LINE_6L",
        "2026 Schedule 3 line 6l: smaller of the negative Form 8978 line 14 magnitude and Form 1040 line 18. Attach Form 8978, its Schedule A, Form 8986, and the prior-tax calculation statement.",
    ),
    (
        "EXPLAIN_SCORP_AMT_LOAN_GAIN_ADJUSTMENT",
        "AMT repayment gain minus regular gain from independent adjusted debt bases. Missing AMT records remain unresolved.",
    ),
    (
        "EXPLAIN_SCORP_N_BACKUP_WITHHOLDING",
        "K-1 box 13 code O backup withholding, included once on Form 1040 line 25c; does not reduce ordinary income or QBI.",
    ),
    (
        "EXPLAIN_SCORP_N_CASH_CHARITY",
        "K-1 box 12 code A cash contributions (60%), Schedule A line 11 with your own cash gifts or line 12f without itemizing; they reduce stock basis and are not part of Form 8582 or QBI.",
    ),
    (
        "EXPLAIN_SCORP_N_DISTRIBUTION_N_N",
        "K-1 box 16 code D distributions above stock basis, capital gain on Form 8949 and Schedule D with the holding period of the shares; not corporate operating income or QBI.",
    ),
    (
        "EXPLAIN_SCORP_N_LOAN_N_N",
        "Written-note repayment gain on Form 8949 and Schedule D, using the source holding period and independent regular/AMT debt basis. Personal-investment gain, not corporate operating income or QBI.",
    ),
    (
        "EXPLAIN_SCORP_ORDINARY_LOAN_GAIN",
        "Ordinary gain on repayment of a personal-investment open-account shareholder loan, including a former open account. Form 4797 and Schedule 1 line 4; not operating income, self-employment earnings or QBI.",
    ),
    (
        "EXPLAIN_SECTION179_BUSINESS_INCOME_LIMIT",
        "Active ordinary business income/losses, your share of materially participated partnership and S-corporation ordinary income after basis and at-risk limits, and employee wages, without section 179 or half-SE deductions, coordinated with the hypothetical home-office deduction before actual limits are applied.",
    ),
    (
        "EXPLAIN_SECTION179_CARRYFORWARD",
        "Total unused elected cost carried by asset or K-1 to a later year, subject to that year's limits and continued eligibility.",
    ),
    (
        "EXPLAIN_SECTION179_DEDUCTION",
        "Total current section 179 deduction, included once in the originating businesses and K-1 activities (Schedule E line 28 column (j)). Preserve asset allocations with Form 4562.",
    ),
    (
        "EXPLAIN_SECTION179_DOLLAR_LIMIT",
        "2026 maximum $2,560,000, reduced dollar-for-dollar by eligible cost above $4,090,000. For a separate return, this line is your allocated share after combining both spouses' eligible purchases; equal shares apply without matching percentage elections. It is not a per-business limit.",
    ),
    (
        "EXPLAIN_SECTION179_ELIGIBLE_COST",
        "All entered eligible 2026 purchases, including zero elections, determine the shared phaseout; Schedule K-1 section 179 shares count toward the elected total, not the phaseout (Regulations section 1.179-2(b)(3)).",
    ),
    (
        "EXPLAIN_SECTION179_SHARED_DOLLAR_LIMIT",
        "Combined dollar limit after both spouses' eligible purchases when filing separately, before allocating the limit.",
    ),
    (
        "EXPLAIN_SECTION179_SPOUSE_ELIGIBLE_COST",
        "Spouse's eligible purchases for separate-return phaseout only; not added to this return's income or depreciation.",
    ),
    (
        "EXPLAIN_SECTION_N_ORDINARY_LOSS",
        "Ordinary Form 4797 loss, limited across the return to $50,000 ($100,000 joint). Excess remains capital loss. Gains are never netted against this allowance. The ordinary loss is business loss for the NOL worksheet, not self-employment earnings or QBI.",
    ),
    (
        "EXPLAIN_SE_NONFARM_WITH_CLERGY",
        "Schedule C and partnership profit plus ministerial earnings {0,number,currency}, before an approved exemption. Schedule SE line A indicator: {1} (1 means an approved Form 4361 exemption).",
    ),
    (
        "EXPLAIN_SE_COMBINED_WITH_NOTARY",
        "Combined farm and nonfarm profit after exempt notary net profit {0,number,currency} (Exempt—Notary).",
    ),
    (
        "EXPLAIN_SE_EXEMPTION_BOXES",
        "Self-employment tax after exemptions. Taxpayer: Schedule 2 line 4 box {0}, code {1}, Schedule SE line A {2}. Spouse: box {3}, code {4}, line A {5}. Zero means no box or line A. Box 1: Form 4361; box 2: Form 4029; box 3: EN notary or EAS totalization.",
    ),
    (
        "EXPLAIN_SELF_EMPLOYMENT_TAX",
        "Per-filer net earnings, Social Security wage-base coordination and Medicare self-employment tax.",
    ),
    (
        "EXPLAIN_SENIOR_DEDUCTION",
        "Eligible SSN holders age 65 or older: phased deduction per person; unavailable to married separate filers.",
    ),
    (
        "EXPLAIN_SOCIAL_LUMP_SUM_TAXABLE_REDUCTION",
        "Reduction in current taxable benefits from the lump-sum election. Benefits remain in current-year line 6a; earlier returns are not amended. Use the election indicator on the filed return when the reduction is positive.",
    ),
    (
        "EXPLAIN_SOCIAL_SECURITY_NET_REPAYMENTS",
        "Excess repayments after combining all box 5 amounts on this return. Net repayments do not create negative current-year benefit income; amounts of $3,000 or less do not qualify for the repealed miscellaneous itemized deduction.",
    ),
    (
        "EXPLAIN_STATE_REFUND_N_N_ALLOCATED",
        "Prior-year part of one state/local income-tax refund, allocated pro rata by the years in which the tax was paid.",
    ),
    (
        "EXPLAIN_STATE_REFUND_N_N_ALLOCATED_2",
        "2026 part of the same refund, allocated pro rata. It reduces 2026 Schedule A state-income-tax payments and is not 2026 income.",
    ),
    (
        "EXPLAIN_STATE_REFUND_N_N_ALLOCATED_3",
        "2026 part of the same refund, allocated pro rata. It reduces the retained 2026 state-income-tax payment ledger but has no state-income-tax deduction effect under the selected 2026 route and is not 2026 income.",
    ),
    (
        "EXPLAIN_STATE_REFUND_N_N_ALLOCATED_4",
        "One state/local income-tax refund allocated pro rata by the tax paid in this prior federal year.",
    ),
    (
        "EXPLAIN_STATE_REFUND_N_N_NET_INCOME_TAX_PAID",
        "Eligible 2026 state-income-tax payments entered in Deductions after the current-year refund share. The existing SALT election and cap use this net amount; the refund is not subtracted a second time.",
    ),
    (
        "EXPLAIN_STATE_REFUND_N_N_NET_INCOME_TAX_PAID_2",
        "Net 2026 state-income-tax payment after this refund share, retained as source data only. It is not deducted when the standard deduction or general sales tax is selected.",
    ),
    (
        "EXPLAIN_STATE_REFUND_N_N_TAXABLE",
        "Taxable prior-year recovery after this deduction year's Schedule A tax-benefit worksheet; only prior-year taxable shares reach Schedule 1 line 1.",
    ),
    (
        "EXPLAIN_STATE_REFUND_N_N_TAXABLE_2",
        "Taxable share after this payment and deduction year's own Schedule A tax-benefit worksheet.",
    ),
    (
        "EXPLAIN_STOCK_N_CAPITAL_BASIS_LOSS",
        "Capital portion from basis that does not qualify for ordinary loss; any excess over the annual ordinary-loss allowance is added separately.",
    ),
    (
        "EXPLAIN_STOCK_N_ORDINARY_BEFORE_LIMIT",
        "Ordinary-loss portion before the annual allowance, after contributed-property and later-capital-contribution basis rules.",
    ),
    (
        "EXPLAIN_TAXABLE_ALIMONY",
        "Qualifying pre-2019 receipts plus payer recapture, after child-support priority and express new-law modifications. Decree-related amounts enter the applicable filer's IRA compensation; neither amount is EIC or SE earnings.",
    ),
    (
        "EXPLAIN_TAXABLE_CASUALTY_LIVING_INSURANCE",
        "Insurer-paid temporary main-home living expenses above the temporary increase, when the casualty did not occur in a federally declared disaster area. Schedule 1 line 8z; not wages, self-employment earnings or net investment income.",
    ),
    (
        "EXPLAIN_TAXABLE_GRANTS_1099G",
        "Taxable grants (Form 1099-G box 6) on Schedule 1 line 8z; not wages, self-employment earnings, EIC or IRA compensation.",
    ),
    ("EXPLAIN_TAXABLE_INCOME", "Taxable income is floored at zero."),
    (
        "EXPLAIN_TAXABLE_LTC_BENEFITS",
        "Taxable qualified periodic LTC and chronic-illness accelerated death benefits after shared per-period exclusions. Schedule 1 income, not wages, self-employment earnings or net investment income.",
    ),
    (
        "EXPLAIN_TAXABLE_HEALTH_LOAN_ASSISTANCE",
        "Loan repayment assistance from a health-professional service program (the National Health Service Corps or a state program) that is not confirmed as excluded under section 108(f)(4). Pub. 970 treats it as other income on Schedule 1 line 8z, not canceled debt.",
    ),
    (
        "EXPLAIN_TAXABLE_PRIZES",
        "Personal nonservice cash and noncash prizes on Schedule 1 line 8i, excluding refused and qualifying directly transferred awards. Not EIC, IRA or self-employment earnings; no automatic charitable deduction.",
    ),
    (
        "EXPLAIN_TAXABLE_SCHOLARSHIPS",
        "Non-W-2 scholarships and fellowships on Schedule 1 line 8r. Earned for the dependent standard deduction, not EIC; taxable graduate/postdoctoral non-tuition stipends also supply IRA compensation.",
    ),
    (
        "EXPLAIN_TAXABLE_SOCIAL_SECURITY",
        "Pub. 915 provisional-income calculation before the student-loan deduction.",
    ),
    (
        "EXPLAIN_TAXABLE_STATE_TAX_REFUND",
        "Schedule 1 line 1 recovery limited to the prior SALT and itemization benefit after allowing the general-sales-tax alternative. Refund interest is separate. This refund is excluded from AMT income and is not earned income.",
    ),
    (
        "EXPLAIN_TAX_CREDIT_BOND_CARRY_N_N",
        "Unused 2026 Form 8912 qualified tax-credit or Build America bond credit attributable to issuer {0} and box 2b identifier {1}. Retain this same-bond balance for 2027 review. It is not a deduction or taxable interest again.",
    ),
    (
        "EXPLAIN_TAX_CREDIT_BOND_CREDIT",
        "Form 8912 line 12 and Schedule 3 line 6k: smaller of total same-bond credit and net income tax after the specified preceding credits.",
    ),
    (
        "EXPLAIN_TAX_CREDIT_BOND_USED_N",
        "Form 8912 line 12 credit attributed to issuer {0} and box 2b identifier {1}. When multiple bonds are partly limited, this is the entered share; otherwise its unique amount is automatic. No IRS bond ordering is assumed.",
    ),
    (
        "EXPLAIN_TIP_FICA_CREDIT_CARRYFORWARD_N",
        "2026 Form 3800 Part IV line 4f column (i): 2025-origin credit remaining for 2027 within its 20-year carryforward period.",
    ),
    (
        "EXPLAIN_TIP_FICA_CURRENT_CREDIT",
        "2026 Form 8846 line 4 and Form 3800 Part III line 4f column (e): current nonpassive employer-tip FICA credit.",
    ),
    (
        "EXPLAIN_TIP_FICA_CURRENT_CREDIT_CARRYBACK_N",
        "Unused 2026 Form 8846 credit applied through a retained 2025 Form 3800 carryback worksheet and Form 1040-X or Form 1045.",
    ),
    (
        "EXPLAIN_TIP_FICA_CURRENT_CREDIT_CARRYFORWARD_N",
        "2026-origin Form 8846 credit remaining for 2027 after 2026 tax use and the 2025 carryback.",
    ),
    (
        "EXPLAIN_TIP_FICA_CURRENT_CREDIT_USED",
        "2026 Form 3800 Part III line 4f: current-year credit used after older carryforwards.",
    ),
    (
        "EXPLAIN_TIP_FICA_N_CARRYBACK_USED",
        "2025-origin credit used on the 2024 carryback return before the 2026 carryforward.",
    ),
    (
        "EXPLAIN_TIP_FICA_N_CREDIT_ORIGIN",
        "2025 Form 8846 line 6 and retained 2025 Form 3800 Part III line 4f specified credit; retain for the required carryforward statement.",
    ),
    (
        "EXPLAIN_TIP_FICA_N_CREDIT_USED",
        "2025 Form 3800 applied credit, retained for the 2026 Part IV carryforward statement.",
    ),
    (
        "EXPLAIN_TOTAL_TAX",
        "Income tax after nonrefundable credits plus self-employment and additional taxes.",
    ),
    (
        "EXPLAIN_TRUMP_ABLE_ROLLOVER_N_GROSS",
        "2026 Form 1099-R code G and Trump-account checkbox: entire account directly transferred to the same beneficiary's ABLE account during the year they turn 17. Box 2a and taxable income are zero; this does not consume the ABLE annual contribution limit.",
    ),
    (
        "EXPLAIN_TRUMP_ACCOUNT_N_N_LINE57_EXCESS",
        "Form 5329 line 57: nonexempt 2026 contributions above $5,000, before completed corrective returns.",
    ),
    (
        "EXPLAIN_TRUMP_ACCOUNT_N_N_LINE59_RETURNED",
        "Form 5329 line 59: excess principal returned by the 2026-return deadline; attributable income is not principal.",
    ),
    (
        "EXPLAIN_TRUMP_ACCOUNT_N_N_LINE60_REMAINING",
        "Form 5329 line 60: excess principal remaining after timely returns.",
    ),
    (
        "EXPLAIN_TRUMP_ACCOUNT_N_N_LINE61_TAX",
        "Form 5329 line 61 and Schedule 2 line 18: 6% tax on remaining 2026 excess contributions.",
    ),
    (
        "EXPLAIN_TRUMP_ACCOUNT_N_N_LINE62_EARNINGS",
        "Form 5329 line 62: positive income attributable to excess returned during 2026, from Form 1099-R box 7d. Later returns belong to their receipt year.",
    ),
    (
        "EXPLAIN_TRUMP_ACCOUNT_N_N_LINE63_TAX",
        "Form 5329 line 63 and Schedule 2 line 5: 100% additional tax on 2026 returned-excess income; the corrective distribution is not ordinary gross income.",
    ),
    (
        "EXPLAIN_TRUMP_DEATH_N_TAXABLE",
        "Form 1099-R box 2a: death-date account value above trustee-reported basis. Shown provisionally as other income because the account ceases to be an IRA on death; no ordinary IRA contribution deduction or early-distribution tax. This is unearned income and can affect a young recipient's Form 8615.",
    ),
    (
        "EXPLAIN_TRUMP_DEATH_N_VALUE",
        "Form 1099-R box 1: the qualified Trump account is deemed distributed at its beneficiary's death during the growth period and ceases to be an IRA. Report on the acquirer's return, or the beneficiary's final return if the estate acquired it.",
    ),
    (
        "EXPLAIN_TRUMP_DEATH_TAXABLE_INCOME",
        "Section 530A(d)(6) income on the qualified beneficiary's death during the growth period: death-date Form 1099-R value less basis, reported once as other income of the acquirer or the deceased beneficiary when the estate acquired the account. The account is no longer an IRA at death; final 2026 Form 1040 line-placement instructions remain pending.",
    ),
    (
        "EXPLAIN_UNDISTRIBUTED_GAIN_TAX_CREDIT",
        "Form 2439 box 2 tax paid by the fund or REIT is a Schedule 3 payment credit, not federal withholding and not limited to this return's tax.",
    ),
    (
        "EXPLAIN_UNEMPLOYMENT_N_N_AGRICULTURE_PAYMENT",
        "Form 1099-G box 7 agricultural program payment, added to this owner's Schedule F fully taxable agricultural-program income (line 4a/4b).",
    ),
    (
        "EXPLAIN_UNEMPLOYMENT_N_N_FAMILY_LEAVE",
        "Paid family leave benefits (Form 1099-G box 10) on Schedule 1 line 8z under Rev. Rul. 2025-4, included only when no medical-leave portion is present.",
    ),
    (
        "EXPLAIN_UNEMPLOYMENT_N_N_GROSS",
        "Gross governmental unemployment benefits before repayments. These benefits are not wages, self-employment earnings or IRA compensation.",
    ),
    (
        "EXPLAIN_UNEMPLOYMENT_N_N_REPAID",
        "2026 benefits actually repaid in 2026. Show the repayment with Schedule 1 line 7; do not claim a second deduction or reduce recorded withholding.",
    ),
    (
        "EXPLAIN_UNEMPLOYMENT_N_N_TAXABLE",
        "Schedule 1 line 7 taxable benefits after same-year repayments. Earlier-year repayments are not netted against current receipts.",
    ),
    (
        "EXPLAIN_UNEMPLOYMENT_N_N_TAXABLE_GRANTS",
        "Taxable grants (Form 1099-G box 6) on Schedule 1 line 8z.",
    ),
    (
        "EXPLAIN_UNEMPLOYMENT_PRIOR_YEAR_REPAYMENTS",
        "Earlier-year benefits repaid in 2026 do not reduce current benefit income. With supported return-wide repayments of $3,000 or less, no deduction or section 1341 credit is allowed in 2026.",
    ),
    (
        "EXPLAIN_UNRECAPTURED_SECTION_N_GAIN",
        "Payer-reported section 1250 gain after capital-loss coordination, capped by net capital gain. The regular-tax rate is at most 25%, not a flat 25%.",
    ),
    (
        "EXPLAIN_UNRECOVERED_ANNUITY_DEDUCTION",
        "Other itemized deduction for contract investment unrecovered at the final annuitant's death, after current-year recovery and other tax-free refunds. Subject to the return's general itemized-deduction limitation.",
    ),
    (
        "EXPLAIN_VARIABLE_ANNUITY_N_ELECTION",
        "Election statement required under Regulations section 1.72-4(d)(3): annuity starting date {0,date}, age at that date {1}; first day of current payment period {2,date}; original investment shown here; prior tax-free recovery {3,number,currency}. State that the tax-free amount is being refigured. This App displays the calculation and does not file the election.",
    ),
    (
        "EXPLAIN_VARIABLE_ANNUITY_N_N_ANNUAL_EXCLUSION",
        "Annual tax-free amount from original cost and expected payments, or the retained worksheet plus an elected shortfall adjustment; retain for next year.",
    ),
    (
        "EXPLAIN_VARIABLE_ANNUITY_N_N_BASIS_RECOVERY",
        "Annual exclusion prorated for the scheduled installments in a first or final year, limited to total payments received and unrecovered investment.",
    ),
    (
        "EXPLAIN_VARIABLE_ANNUITY_N_N_EARLIER_RECOVERY",
        "Tax-free recovery from the first annuitant's earlier current-year installments, capped by those receipts and common remaining investment. Not survivor income.",
    ),
    (
        "EXPLAIN_VARIABLE_ANNUITY_N_N_REMAINING_BASIS",
        "Unrecovered cost after earlier first-annuitant and current recipient recovery; retain the common record for the next worksheet.",
    ),
    (
        "EXPLAIN_W2_UNCOLLECTED_PAYROLL_TAX",
        "W-2 box 12 codes A/B/M/N enter Schedule 2 as tax due, not additional income or withholding.",
    ),
    (
        "EXPLAIN_WAGES",
        "Form 8958 community-allocated W-2 box 1 wages, then round the tax line. The worker's personal wages remain separate for payroll, IRA and EIC calculations.",
    ),
    ("EXPLAIN_WAGES_2", "Aggregate W-2 box 1 amounts, then round the tax line."),
    (
        "EXPLAIN_WILDFIRE_RELIEF_N_EXCLUDED",
        "2026 section 139M qualified wildfire relief excluded from gross income. The exclusion does not allow a second deduction, credit or property-basis increase for compensated losses.",
    ),
    (
        "EXPLAIN_WORTHLESS_SECURITY_N_LOSS",
        "Long-term capital loss on a deemed December 31, 2026 sale. Ordinary inherited capital assets are long-term regardless of acquisition date. Netting and annual capital-loss limits apply.",
    ),
    (
        "EXPLAIN_WORTHLESS_SECURITY_N_LOSS_2",
        "Short-term capital loss on a deemed December 31, 2026 sale, measured from the day after the acquisition trade date. Netting and annual capital-loss limits apply.",
    ),
    ("DISPLAY_W2_ANSWERS", "Your answers"),
    (
        "W2_TP_OCCUPATION_NEEDED",
        "Answer whether these tips are from an IRS-listed occupation when code TP, box 8 or daily tip records are entered.",
    ),
    (
        "DIV_REIT_ELIGIBILITY_NEEDED",
        "Box 5 shows section 199A dividends: answer whether they satisfy the holding-period and payment-obligation rules.",
    ),
    ("W2_BOX14_KIND_NEEDED", "Say what each box 14 item on this W-2 is."),
    (
        "W2_BOX14_STATE_NEEDED",
        "Choose the state of each box 14 state disability, family-leave or pickup item.",
    ),
    (
        "W2_TT_FLSA_NEEDED",
        "Box 12 shows code TT: answer whether this is overtime pay the Fair Labor Standards Act requires.",
    ),
    ("EDUCATION_METHOD_NEEDED", "Choose which education credit to claim"),
    (
        "EDUCATION_AOTC_HISTORY_NEEDED",
        "Answer the prior education credit and student eligibility question.",
    ),
    ("LINE_CHILD_ELECTION_N_INTEREST", "Child election {0} interest (line 1a)"),
    (
        "EXPLAIN_CHILD_ELECTION_N_INTEREST",
        "Child's taxable interest after bond premium and accrued interest, from the child's Forms 1099-INT and 1099-OID. It counts toward line 4 only and is not Form 1040 line 2b.",
    ),
    ("LINE_CHILD_ELECTION_N_EXEMPT_INTEREST", "Child election {0} tax-exempt interest (line 1b)"),
    (
        "EXPLAIN_CHILD_ELECTION_N_EXEMPT_INTEREST",
        "Child's tax-exempt interest. Its private-activity bond part is an alternative minimum tax preference, and the amount counts toward the earned income credit's investment income.",
    ),
    ("LINE_CHILD_ELECTION_N_ORDINARY_DIVIDENDS", "Child election {0} ordinary dividends (line 2a)"),
    (
        "EXPLAIN_CHILD_ELECTION_N_ORDINARY_DIVIDENDS",
        "Child's ordinary dividends from Forms 1099-DIV box 1a, plus the Alaska Permanent Fund dividend.",
    ),
    (
        "LINE_CHILD_ELECTION_N_QUALIFIED_DIVIDENDS",
        "Child election {0} qualified dividends (line 2b)",
    ),
    (
        "EXPLAIN_CHILD_ELECTION_N_QUALIFIED_DIVIDENDS",
        "The part of line 2a that is qualified dividends.",
    ),
    (
        "LINE_CHILD_ELECTION_N_CAPITAL_GAIN_DISTRIBUTIONS",
        "Child election {0} capital gain distributions (line 3)",
    ),
    (
        "EXPLAIN_CHILD_ELECTION_N_CAPITAL_GAIN_DISTRIBUTIONS",
        "Child's capital gain distributions from Forms 1099-DIV box 2a.",
    ),
    ("LINE_CHILD_ELECTION_N_TOTAL_INCOME", "Child election {0} total (line 4)"),
    (
        "EXPLAIN_CHILD_ELECTION_N_TOTAL_INCOME",
        "Lines 1a, 2a and 3. Over $1,350 and under $13,500, the child can be reported on your return.",
    ),
    ("LINE_CHILD_ELECTION_N_INCOME_OVER_BASE", "Child election {0} income over $2,700 (line 6)"),
    (
        "EXPLAIN_CHILD_ELECTION_N_INCOME_OVER_BASE",
        "Line 4 less $2,700. The first $2,700 is not added to your income; this excess is.",
    ),
    (
        "LINE_CHILD_ELECTION_N_QUALIFIED_DIVIDENDS_REPORTED",
        "Child election {0} qualified dividends reported (line 9)",
    ),
    (
        "EXPLAIN_CHILD_ELECTION_N_QUALIFIED_DIVIDENDS_REPORTED",
        "Line 6 times the fraction of line 4 that is qualified dividends. It goes on Form 1040 lines 3a and 3b.",
    ),
    (
        "LINE_CHILD_ELECTION_N_CAPITAL_GAIN_DISTRIBUTIONS_REPORTED",
        "Child election {0} capital gain distributions reported (line 10)",
    ),
    (
        "EXPLAIN_CHILD_ELECTION_N_CAPITAL_GAIN_DISTRIBUTIONS_REPORTED",
        "Line 6 times the fraction of line 4 that is capital gain distributions. It goes on Schedule D line 13.",
    ),
    (
        "LINE_CHILD_ELECTION_N_REPORTED_TOTAL",
        "Child election {0} dividends and distributions reported (line 11)",
    ),
    ("EXPLAIN_CHILD_ELECTION_N_REPORTED_TOTAL", "Lines 9 and 10."),
    ("LINE_CHILD_ELECTION_N_OTHER_INCOME", "Child election {0} other income (line 12)"),
    (
        "EXPLAIN_CHILD_ELECTION_N_OTHER_INCOME",
        "Line 6 less line 11: the child's interest and other dividends above $2,700. It goes on Schedule 1 line 8z.",
    ),
    ("LINE_CHILD_ELECTION_N_TAXABLE_OVER_UNTAXED", "Child election {0} over $1,350 (line 14)"),
    ("EXPLAIN_CHILD_ELECTION_N_TAXABLE_OVER_UNTAXED", "Line 4 less $1,350, the amount not taxed."),
    ("LINE_CHILD_ELECTION_N_TAX", "Child election {0} tax (line 15)"),
    (
        "EXPLAIN_CHILD_ELECTION_N_TAX",
        "10% of line 14, at most $135. It is added to Form 1040 line 16, even for income the child's own rate would tax less.",
    ),
    ("LINE_CHILD_ELECTION_QUALIFIED_DIVIDENDS", "Qualified dividends reported for children"),
    (
        "EXPLAIN_CHILD_ELECTION_QUALIFIED_DIVIDENDS",
        "Form 8814 line 9 of every child. It is inside Form 1040 lines 3a and 3b.",
    ),
    (
        "LINE_CHILD_ELECTION_CAPITAL_GAIN_DISTRIBUTIONS",
        "Capital gain distributions reported for children",
    ),
    (
        "EXPLAIN_CHILD_ELECTION_CAPITAL_GAIN_DISTRIBUTIONS",
        "Form 8814 line 10 of every child. It is inside Schedule D line 13 and Form 1040 line 7a.",
    ),
    ("LINE_CHILD_ELECTION_OTHER_INCOME", "Other income reported for children"),
    (
        "EXPLAIN_CHILD_ELECTION_OTHER_INCOME",
        "Form 8814 line 12 of every child. It is Schedule 1 line 8z.",
    ),
    ("LINE_CHILD_ELECTION_INVESTMENT_INCOME", "Investment income reported for children"),
    (
        "EXPLAIN_CHILD_ELECTION_INVESTMENT_INCOME",
        "Form 8814 line 12 of every child less the Alaska Permanent Fund share. It is Form 8960 line 7, Form 4952 investment income and the earned income credit's investment income.",
    ),
    ("LINE_CHILD_ELECTION_EXEMPT_INTEREST", "Tax-exempt interest reported for children"),
    (
        "EXPLAIN_CHILD_ELECTION_EXEMPT_INTEREST",
        "Form 8814 line 1b of every child. It counts toward the earned income credit's investment income and not Form 1040 line 2a.",
    ),
    ("LINE_CHILD_ELECTION_TAX", "Tax on children's income"),
    (
        "EXPLAIN_CHILD_ELECTION_TAX",
        "Form 8814 line 15 of every child, added to Form 1040 line 16. It is not reduced by credits.",
    ),
    ("MESSAGE_ENTER_AT_MOST_20_RELATED_EXCHANGES", "Enter at most 20 related-party exchanges."),
    (
        "EXPLAIN_EXCHANGE_LINE15",
        "Cash you received, plus the value of other property you received, plus loans the other side took over from you net of the loans you took on and the cash you paid, less your exchange expenses (not below zero).",
    ),
    ("EXPLAIN_EXCHANGE_LINE16", "Fair market value of the like-kind property you received."),
    ("EXPLAIN_EXCHANGE_LINE17", "Lines 15 and 16 added."),
    (
        "EXPLAIN_EXCHANGE_LINE18",
        "Adjusted basis of the rental you gave up (land and buildings less depreciation allowed or allowable through the month you gave it up), plus the net loans you took on and cash you paid, plus the exchange expenses not used on line 15.",
    ),
    ("EXPLAIN_EXCHANGE_LINE19", "Realized gain or (loss): line 17 less line 18."),
    (
        "EXPLAIN_EXCHANGE_LINE20",
        "The smaller of line 15 or line 19, not below zero: the gain recognized because you also received cash or other property. A loss is never recognized.",
    ),
    (
        "EXPLAIN_EXCHANGE_LINE21",
        "Ordinary income under the recapture rules, always zero here: a building depreciated straight line after 1986 has no additional depreciation to recapture (Form 4797 line 17 in the 2026 draft).",
    ),
    (
        "EXPLAIN_EXCHANGE_LINE22",
        "Line 20 less line 21: section 1231 gain on property held more than one year, to Form 4797 line 5.",
    ),
    ("EXPLAIN_EXCHANGE_LINE23", "Recognized gain: lines 21 and 22 added."),
    (
        "EXPLAIN_EXCHANGE_LINE24",
        "Deferred gain or (loss): line 19 less line 23. It reduces the basis of the property you received.",
    ),
    (
        "EXPLAIN_EXCHANGE_LINE25",
        "Basis of the like-kind property you received: line 18 plus line 23 less line 15, which is its value less the deferred gain.",
    ),
    (
        "EXPLAIN_EXCHANGE_LINE25A",
        "Part of that basis in the building, in proportion to values. The part up to the adjusted depreciable basis of the building you gave up continues its depreciation; any excess is depreciated as new property.",
    ),
    (
        "EXPLAIN_EXCHANGE_SECTION1231",
        "Section 1231 gain recognized in a like-kind exchange (Form 8824 line 22), Form 4797 line 5.",
    ),
    (
        "EXPLAIN_RELATED_EXCHANGE_GAIN",
        "Deferred gain from an earlier related-party exchange, taxable in 2026 because the related person disposed of the property within two years (Form 8824 line 24).",
    ),
    (
        "EXPLAIN_RENTAL_REPLACEMENT_CARRYOVER_DEPRECIATION",
        "Depreciation of the exchanged basis of the property you received in a like-kind exchange: the same method over the remaining recovery period of the building you gave up (Form 4562 line 17; Reg. 1.168(i)-6).",
    ),
    (
        "EXPLAIN_RENTAL_REPLACEMENT_EXCESS_DEPRECIATION",
        "Depreciation of the excess basis of the property you received in a like-kind exchange, placed in service the day it was ready to rent (Form 4562 lines 19i and 19j; Reg. 1.168(i)-6(d)).",
    ),
    ("LINE_RENTAL_N_EXCHANGE_LINE15", "Rental {0} exchange line 15"),
    ("LINE_RENTAL_N_EXCHANGE_LINE16", "Rental {0} exchange line 16"),
    ("LINE_RENTAL_N_EXCHANGE_LINE17", "Rental {0} exchange line 17"),
    ("LINE_RENTAL_N_EXCHANGE_LINE18", "Rental {0} exchange line 18"),
    ("LINE_RENTAL_N_EXCHANGE_LINE19", "Rental {0} exchange line 19"),
    ("LINE_RENTAL_N_EXCHANGE_LINE20", "Rental {0} exchange line 20"),
    ("LINE_RENTAL_N_EXCHANGE_LINE21", "Rental {0} exchange line 21"),
    ("LINE_RENTAL_N_EXCHANGE_LINE22", "Rental {0} exchange line 22"),
    ("LINE_RENTAL_N_EXCHANGE_LINE23", "Rental {0} exchange line 23"),
    ("LINE_RENTAL_N_EXCHANGE_LINE24", "Rental {0} exchange line 24"),
    ("LINE_RENTAL_N_EXCHANGE_LINE25", "Rental {0} exchange line 25"),
    ("LINE_RENTAL_N_EXCHANGE_LINE25A", "Rental {0} exchange line 25a"),
    ("LINE_RENTAL_N_EXCHANGE_SECTION1231", "Rental {0} exchange section 1231 gain"),
    ("LINE_RENTAL_N_EXCHANGE_UNRECAPTURED_N", "Rental {0} exchange unrecaptured {1}"),
    (
        "LINE_RENTAL_N_REPLACEMENT_CARRYOVER_DEPRECIATION",
        "Rental {0} replacement carryover depreciation",
    ),
    ("LINE_RENTAL_N_REPLACEMENT_EXCESS_DEPRECIATION", "Rental {0} replacement excess depreciation"),
    ("LINE_RELATED_EXCHANGE_N_GAIN", "Related-party exchange {0} deferred gain"),
    ("LINE_RELATED_EXCHANGE_N_SECTION1231", "Related-party exchange {0} section 1231 gain"),
    ("LINE_RELATED_EXCHANGE_N_UNRECAPTURED_N", "Related-party exchange {0} unrecaptured {1}"),
    (
        "LINE_ACCOUNT_TAXPAYER_EXCEPTIONS",
        "Account distributions, taxpayer, not subject to the additional tax",
    ),
    (
        "LINE_ACCOUNT_SPOUSE_EXCEPTIONS",
        "Account distributions, spouse, not subject to the additional tax",
    ),
    (
        "LINE_ACCOUNT_TAXPAYER_SUBJECT",
        "Account distributions, taxpayer, subject to the additional tax",
    ),
    ("LINE_ACCOUNT_SPOUSE_SUBJECT", "Account distributions, spouse, subject to the additional tax"),
    ("LINE_ACCOUNT_TAXPAYER_TAX", "Account distributions, taxpayer, additional tax"),
    ("LINE_ACCOUNT_SPOUSE_TAX", "Account distributions, spouse, additional tax"),
    ("LINE_ACCOUNT_TAXPAYER_TAXABLE", "Account distributions, taxpayer, included in income"),
    ("LINE_ACCOUNT_SPOUSE_TAXABLE", "Account distributions, spouse, included in income"),
    ("LINE_ACCOUNT_ADDITIONAL_TAX", "ABLE and education account additional tax"),
    (
        "LINE_EDUCATION_ACCOUNT_N_ADJUSTED_EXPENSES",
        "Education account {0} adjusted qualified expenses",
    ),
    ("LINE_EDUCATION_ACCOUNT_N_BASIS_REMAINING", "Education account {0} basis at year end"),
    ("LINE_EDUCATION_ACCOUNT_N_DISTRIBUTION", "Education account {0} distribution"),
    ("LINE_EDUCATION_ACCOUNT_N_EARNINGS", "Education account {0} earnings"),
    (
        "LINE_EDUCATION_ACCOUNT_N_EXCEPTIONS",
        "Education account {0} earnings not subject to the additional tax",
    ),
    ("LINE_EDUCATION_ACCOUNT_N_TAXABLE", "Education account {0} taxable earnings"),
    (
        "LINE_EDUCATION_ACCOUNT_LOAN_INTEREST_REMOVED",
        "Student loan interest paid with tax-free 529 earnings",
    ),
    ("LINE_EDUCATION_ACCOUNT_TAXABLE_EARNINGS", "Taxable 529 and Coverdell earnings"),
    (
        "EXPLAIN_ACCOUNT_EXCEPTIONS",
        "Form 5329 Part II line 6: the distributions on line 5 that are not subject to the additional tax (death, disability, a timely returned Coverdell excess, expenses used only for an education credit, tax-free scholarships and military academy costs, and ABLE distributions not penalized).",
    ),
    ("EXPLAIN_ACCOUNT_SUBJECT", "Form 5329 Part II line 7: line 5 less line 6."),
    (
        "EXPLAIN_ACCOUNT_TAX",
        "Form 5329 Part II line 8: 10% of line 7, carried to Schedule 2 line 5.",
    ),
    (
        "EXPLAIN_ACCOUNT_TAXABLE",
        "Form 5329 Part II line 5: the ABLE and education account distributions included in income, with source amounts added before rounding.",
    ),
    (
        "EXPLAIN_ACCOUNT_ADDITIONAL_TAX",
        "Form 5329 Part II line 8 of each filer added: 10% of the ABLE and education account distributions subject to the additional tax, Schedule 2 line 5.",
    ),
    (
        "EXPLAIN_EDUCATION_ACCOUNT_N_ADJUSTED_EXPENSES",
        "The adjusted qualified education expenses matched with this distribution: the student's qualified expenses less tax-free assistance, the expenses an education credit used and expenses used elsewhere, shared between the student's 529 and Coverdell distributions (Publication 970 Worksheet 6-3 line 5; chapter 7).",
    ),
    (
        "EXPLAIN_EDUCATION_ACCOUNT_N_BASIS_REMAINING",
        "Publication 970 Worksheet 6-3 line 15: the Coverdell account's basis at the end of 2026, to carry to next year's worksheet.",
    ),
    (
        "EXPLAIN_EDUCATION_ACCOUNT_N_DISTRIBUTION",
        "The Form 1099-Q gross distribution less any part put back within 60 days; for a direct transfer to another account, only the part that does not qualify.",
    ),
    (
        "EXPLAIN_EDUCATION_ACCOUNT_N_EARNINGS",
        "The earnings in that distribution: Form 1099-Q box 2 in proportion to the distribution or, for a Coverdell ESA with only a year-end value, Publication 970 Worksheet 6-3 lines 3 to 11.",
    ),
    (
        "EXPLAIN_EDUCATION_ACCOUNT_N_EXCEPTIONS",
        "The taxable earnings not subject to the 10% additional tax: death or disability, a timely returned Coverdell excess, expenses used only for an education credit, and the part covered by tax-free scholarships or military academy costs (Form 5329 Part II line 6).",
    ),
    (
        "EXPLAIN_EDUCATION_ACCOUNT_N_TAXABLE",
        "The earnings left taxable: earnings less the part matched by adjusted qualified expenses (Publication 970 Worksheet 6-3 line 14; chapter 7 step 2), Schedule 1 line 8z. A loss is never deducted.",
    ),
    (
        "EXPLAIN_EDUCATION_ACCOUNT_LOAN_INTEREST_REMOVED",
        "Earnings made tax-free by 529 payments on this student's own student loans, which cannot also be deducted as student loan interest (section 221(e)(1)); removed before the income phaseout.",
    ),
    (
        "EXPLAIN_EDUCATION_ACCOUNT_TAXABLE_EARNINGS",
        "The taxable earnings of every Form 1099-Q from a 529 plan or Coverdell ESA: Schedule 1 line 8z.",
    ),
    // Form 4835 calculation labels and explanations.
    (
        "LINE_FARM_RENTAL_N_PRODUCTION",
        "Farm rental {0} crop and livestock shares converted to money",
    ),
    (
        "EXPLAIN_FARM_RENTAL_N_PRODUCTION",
        "Form 4835 line 1: crop and livestock shares sold in 2026 or converted to money, with the net of a redeemed CCC commodity that was sold.",
    ),
    ("LINE_FARM_RENTAL_N_COOPERATIVE_DISTRIBUTIONS", "Farm rental {0} cooperative distributions"),
    (
        "EXPLAIN_FARM_RENTAL_N_COOPERATIVE_DISTRIBUTIONS",
        "Form 4835 line 2a: Form 1099-PATR distributions routed to this rental.",
    ),
    ("LINE_FARM_RENTAL_N_COOPERATIVE_TAXABLE", "Farm rental {0} taxable cooperative distributions"),
    (
        "EXPLAIN_FARM_RENTAL_N_COOPERATIVE_TAXABLE",
        "Form 4835 line 2b: the taxable part of those distributions.",
    ),
    ("LINE_FARM_RENTAL_N_PROGRAM_PAYMENTS", "Farm rental {0} agricultural program payments"),
    (
        "EXPLAIN_FARM_RENTAL_N_PROGRAM_PAYMENTS",
        "Form 4835 line 3a: agricultural program payments, Form 1099-G box 7 payments routed to this rental and CCC market gain.",
    ),
    ("LINE_FARM_RENTAL_N_PROGRAM_TAXABLE", "Farm rental {0} taxable agricultural program payments"),
    (
        "EXPLAIN_FARM_RENTAL_N_PROGRAM_TAXABLE",
        "Form 4835 line 3b: the taxable part of those payments.",
    ),
    ("LINE_FARM_RENTAL_N_OTHER_INCOME", "Farm rental {0} other income"),
    (
        "EXPLAIN_FARM_RENTAL_N_OTHER_INCOME",
        "Form 4835 line 6: other income, including any CCC surplus above the loan.",
    ),
    ("LINE_FARM_RENTAL_N_GROSS_INCOME", "Farm rental {0} gross farm rental income"),
    (
        "EXPLAIN_FARM_RENTAL_N_GROSS_INCOME",
        "Form 4835 line 7: gross farm rental income, lines 1, 2b, 3b, 4a, 4c, 5b, 5d and 6, which is also Schedule E line 42 income.",
    ),
    ("LINE_FARM_RENTAL_N_DEPRECIATION", "Farm rental {0} depreciation"),
    (
        "EXPLAIN_FARM_RENTAL_N_DEPRECIATION",
        "Form 4835 line 12: depreciation of this rental's property. Section 179 is not figured for a farm rental.",
    ),
    ("LINE_FARM_RENTAL_N_EXPENSES", "Farm rental {0} total expenses"),
    (
        "EXPLAIN_FARM_RENTAL_N_EXPENSES",
        "Form 4835 line 31: total expenses, including the section 175 conservation deduction and any CCC deficiency paid.",
    ),
    ("LINE_FARM_RENTAL_N_NET_INCOME", "Farm rental {0} net income or loss"),
    (
        "EXPLAIN_FARM_RENTAL_N_NET_INCOME",
        "Form 4835 line 32: net farm rental income or loss. A rental that is income only before its prior-year unallowed losses shows zero (PAL); the loss goes to Form 8582 and line 34c.",
    ),
    ("LINE_FARM_RENTAL_N_LINE34C_ALLOWED_LOSS", "Farm rental {0} allowed loss"),
    (
        "EXPLAIN_FARM_RENTAL_N_LINE34C_ALLOWED_LOSS",
        "Form 4835 line 34c: the loss allowed after Form 6198 and Form 8582, which goes to Schedule E line 40.",
    ),
    (
        "LINE_PARTNERSHIP_N_RENTAL_REAL_ESTATE_NET",
        "Partnership {0} rental real estate after passive limits",
    ),
    (
        "LINE_PARTNERSHIP_N_RENTAL_REAL_ESTATE_SUSPENDED",
        "Partnership {0} rental real estate suspended loss",
    ),
    ("LINE_PARTNERSHIP_N_OTHER_RENTAL_NET", "Partnership {0} other rental after passive limits"),
    ("LINE_PARTNERSHIP_N_OTHER_RENTAL_SUSPENDED", "Partnership {0} other rental suspended loss"),
    (
        "LINE_AMT_PARTNERSHIP_N_RENTAL_REAL_ESTATE_NET",
        "AMT partnership {0} rental real estate after passive limits",
    ),
    (
        "LINE_AMT_PARTNERSHIP_N_RENTAL_REAL_ESTATE_SUSPENDED",
        "AMT partnership {0} rental real estate suspended loss",
    ),
    (
        "LINE_AMT_PARTNERSHIP_N_OTHER_RENTAL_NET",
        "AMT partnership {0} other rental after passive limits",
    ),
    (
        "LINE_AMT_PARTNERSHIP_N_OTHER_RENTAL_SUSPENDED",
        "AMT partnership {0} other rental suspended loss",
    ),
    (
        "LINE_SCORP_N_RENTAL_REAL_ESTATE_NET",
        "S corporation {0} rental real estate after passive limits",
    ),
    (
        "LINE_SCORP_N_RENTAL_REAL_ESTATE_SUSPENDED",
        "S corporation {0} rental real estate suspended loss",
    ),
    ("LINE_SCORP_N_OTHER_RENTAL_NET", "S corporation {0} other rental after passive limits"),
    ("LINE_SCORP_N_OTHER_RENTAL_SUSPENDED", "S corporation {0} other rental suspended loss"),
    (
        "LINE_AMT_SCORP_N_RENTAL_REAL_ESTATE_NET",
        "AMT S corporation {0} rental real estate after passive limits",
    ),
    (
        "LINE_AMT_SCORP_N_RENTAL_REAL_ESTATE_SUSPENDED",
        "AMT S corporation {0} rental real estate suspended loss",
    ),
    (
        "LINE_AMT_SCORP_N_OTHER_RENTAL_NET",
        "AMT S corporation {0} other rental after passive limits",
    ),
    (
        "LINE_AMT_SCORP_N_OTHER_RENTAL_SUSPENDED",
        "AMT S corporation {0} other rental suspended loss",
    ),
    (
        "EXPLAIN_K1_RENTAL_REAL_ESTATE_NET",
        "Schedule E line 28: this K-1's box 2 rental real estate income or loss after the basis, at-risk and Form 8582 limits; column (g) or (h) when passive, (i) or (k) when a real estate professional materially participated.",
    ),
    (
        "EXPLAIN_K1_RENTAL_REAL_ESTATE_SUSPENDED",
        "Box 2 rental real estate loss that Form 8582 does not allow in 2026; it carries to 2027 with this activity.",
    ),
    (
        "EXPLAIN_K1_OTHER_RENTAL_NET",
        "Schedule E line 28: this K-1's box 3 other rental income or loss after the basis, at-risk and Form 8582 limits, column (g) or (h).",
    ),
    (
        "EXPLAIN_K1_OTHER_RENTAL_SUSPENDED",
        "Box 3 other rental loss that Form 8582 does not allow in 2026; it carries to 2027 with this activity.",
    ),
    (
        "EXPLAIN_AMT_K1_RENTAL_REAL_ESTATE_NET",
        "Alternative minimum tax: this K-1's box 2 rental real estate income or loss after that system's basis, at-risk and Form 8582 limits.",
    ),
    (
        "EXPLAIN_AMT_K1_RENTAL_REAL_ESTATE_SUSPENDED",
        "Alternative minimum tax box 2 rental real estate loss that Form 8582 does not allow in 2026; it carries to 2027.",
    ),
    (
        "EXPLAIN_AMT_K1_OTHER_RENTAL_NET",
        "Alternative minimum tax: this K-1's box 3 other rental income or loss after that system's basis, at-risk and Form 8582 limits.",
    ),
    (
        "EXPLAIN_AMT_K1_OTHER_RENTAL_SUSPENDED",
        "Alternative minimum tax box 3 other rental loss that Form 8582 does not allow in 2026; it carries to 2027.",
    ),
    ("LINE_FARM_RENTAL_N_SUSPENDED_LOSS", "Farm rental {0} suspended loss"),
    (
        "EXPLAIN_FARM_RENTAL_N_SUSPENDED_LOSS",
        "Passive loss of this farm rental that Form 8582 does not allow in 2026; it carries to 2027.",
    ),
    ("LINE_FARM_RENTAL_N_CAR_AND_TRUCK", "Farm rental {0} car and truck expenses"),
    (
        "EXPLAIN_FARM_RENTAL_N_CAR_AND_TRUCK",
        "Form 4835 line 8: car and truck expenses of this rental's vehicles, without the vehicle-loan interest shown on line 19b.",
    ),
    ("LINE_FARM_RENTAL_N_VEHICLE_LOAN_INTEREST", "Farm rental {0} vehicle loan interest"),
    (
        "EXPLAIN_FARM_RENTAL_N_VEHICLE_LOAN_INTEREST",
        "Form 4835 line 19b: the business share of interest on loans for this rental's vehicles.",
    ),
    (
        "LINE_FARM_RENTAL_N_EXPENSE_LINE13_EMPLOYEE_BENEFIT_PROGRAMS",
        "Farm rental {0} line 13 employee benefit programs",
    ),
    (
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE13_EMPLOYEE_BENEFIT_PROGRAMS",
        "Form 4835 line 13: employee benefit programs other than on line 21.",
    ),
    ("LINE_FARM_RENTAL_N_EXPENSE_LINE18_INSURANCE", "Farm rental {0} line 18 insurance"),
    (
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE18_INSURANCE",
        "Form 4835 line 18: insurance other than health.",
    ),
    (
        "LINE_FARM_RENTAL_N_EXPENSE_LINE19A_MORTGAGE_INTEREST",
        "Farm rental {0} line 19a mortgage interest",
    ),
    (
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE19A_MORTGAGE_INTEREST",
        "Form 4835 line 19a: mortgage interest paid to banks.",
    ),
    (
        "LINE_FARM_RENTAL_N_EXPENSE_LINE19C_OTHER_INTEREST",
        "Farm rental {0} line 19c other interest",
    ),
    ("EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE19C_OTHER_INTEREST", "Form 4835 line 19c: other interest."),
    (
        "LINE_FARM_RENTAL_N_EXPENSE_LINE22A_RENT_VEHICLES_EQUIPMENT",
        "Farm rental {0} line 22a rent of vehicles and equipment",
    ),
    (
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE22A_RENT_VEHICLES_EQUIPMENT",
        "Form 4835 line 22a: rent or lease of vehicles, machinery and equipment.",
    ),
    (
        "LINE_FARM_RENTAL_N_EXPENSE_LINE22B_RENT_OTHER",
        "Farm rental {0} line 22b rent of other property",
    ),
    (
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE22B_RENT_OTHER",
        "Form 4835 line 22b: rent or lease of other property.",
    ),
    ("LINE_FARM_RENTAL_N_EXPENSE_LINE23_REPAIRS", "Farm rental {0} line 23 repairs"),
    ("EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE23_REPAIRS", "Form 4835 line 23: repairs and maintenance."),
    ("LINE_FARM_RENTAL_N_EXPENSE_LINE26_SUPPLIES", "Farm rental {0} line 26 supplies"),
    ("EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE26_SUPPLIES", "Form 4835 line 26: supplies."),
    ("LINE_FARM_RENTAL_N_EXPENSE_LINE27_TAXES", "Farm rental {0} line 27 taxes"),
    ("EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE27_TAXES", "Form 4835 line 27: taxes."),
    ("LINE_FARM_RENTAL_N_EXPENSE_LINE30_MEALS", "Farm rental {0} line 30 meals"),
    (
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE30_MEALS",
        "Form 4835 line 30: business meals at 50%, one of the other expenses on lines 30a to 30f.",
    ),
    ("LINE_FARM_RENTAL_N_EXPENSE_LINE28_UTILITIES", "Farm rental {0} line 28 utilities"),
    ("EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE28_UTILITIES", "Form 4835 line 28: utilities."),
    ("LINE_FARM_RENTAL_N_EXPENSE_LINE20_LABOR_HIRED", "Farm rental {0} line 20 labor hired"),
    (
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE20_LABOR_HIRED",
        "Form 4835 line 20: labor hired, less employment credits.",
    ),
    ("LINE_FARM_RENTAL_N_EXPENSE_LINE30_OTHER", "Farm rental {0} line 30 other expenses"),
    (
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE30_OTHER",
        "Form 4835 line 30: other expenses, lines 30a to 30f.",
    ),
    ("LINE_FARM_RENTAL_N_EXPENSE_LINE9_CHEMICALS", "Farm rental {0} line 9 chemicals"),
    ("EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE9_CHEMICALS", "Form 4835 line 9: chemicals."),
    ("LINE_FARM_RENTAL_N_EXPENSE_LINE11_CUSTOM_HIRE", "Farm rental {0} line 11 custom hire"),
    (
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE11_CUSTOM_HIRE",
        "Form 4835 line 11: custom hire (machine work).",
    ),
    ("LINE_FARM_RENTAL_N_EXPENSE_LINE14_FEED", "Farm rental {0} line 14 feed"),
    ("EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE14_FEED", "Form 4835 line 14: feed."),
    (
        "LINE_FARM_RENTAL_N_EXPENSE_LINE15_FERTILIZERS_AND_LIME",
        "Farm rental {0} line 15 fertilizers and lime",
    ),
    (
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE15_FERTILIZERS_AND_LIME",
        "Form 4835 line 15: fertilizers and lime.",
    ),
    (
        "LINE_FARM_RENTAL_N_EXPENSE_LINE16_FREIGHT_AND_TRUCKING",
        "Farm rental {0} line 16 freight and trucking",
    ),
    (
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE16_FREIGHT_AND_TRUCKING",
        "Form 4835 line 16: freight and trucking.",
    ),
    (
        "LINE_FARM_RENTAL_N_EXPENSE_LINE17_GASOLINE_FUEL_AND_OIL",
        "Farm rental {0} line 17 gasoline, fuel and oil",
    ),
    (
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE17_GASOLINE_FUEL_AND_OIL",
        "Form 4835 line 17: gasoline, fuel and oil.",
    ),
    (
        "LINE_FARM_RENTAL_N_EXPENSE_LINE21_PENSION_AND_PROFIT_SHARING",
        "Farm rental {0} line 21 pension and profit-sharing plans",
    ),
    (
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE21_PENSION_AND_PROFIT_SHARING",
        "Form 4835 line 21: pension and profit-sharing plans.",
    ),
    (
        "LINE_FARM_RENTAL_N_EXPENSE_LINE24_SEEDS_AND_PLANTS",
        "Farm rental {0} line 24 seeds and plants",
    ),
    (
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE24_SEEDS_AND_PLANTS",
        "Form 4835 line 24: seeds and plants.",
    ),
    (
        "LINE_FARM_RENTAL_N_EXPENSE_LINE25_STORAGE_AND_WAREHOUSING",
        "Farm rental {0} line 25 storage and warehousing",
    ),
    (
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE25_STORAGE_AND_WAREHOUSING",
        "Form 4835 line 25: storage and warehousing.",
    ),
    (
        "LINE_FARM_RENTAL_N_EXPENSE_LINE29_VETERINARY_BREEDING_AND_MEDICINE",
        "Farm rental {0} line 29 veterinary, breeding and medicine",
    ),
    (
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE29_VETERINARY_BREEDING_AND_MEDICINE",
        "Form 4835 line 29: veterinary, breeding and medicine.",
    ),
    ("LINE_AMT_FARM_RENTAL_N_SUSPENDED_LOSS", "AMT farm rental {0} suspended loss"),
    (
        "EXPLAIN_0FARM_RENTAL_N_SUSPENDED_LOSS",
        "Alternative minimum tax passive loss of this farm rental that Form 8582 does not allow in 2026; it carries to 2027.",
    ),
    ("LINE_SCHEDULE_E_FARM_RENTAL_TOTAL", "Schedule E farm rental total"),
    (
        "EXPLAIN_SCHEDULE_E_FARM_RENTAL_TOTAL",
        "Schedule E line 40: net farm rental income or deductible loss from Form 4835.",
    ),
    ("LINE_SCHEDULE_E_FARMING_FISHING_GROSS", "Schedule E farming and fishing gross income"),
    (
        "EXPLAIN_SCHEDULE_E_FARMING_FISHING_GROSS",
        "Schedule E line 42: gross farming and fishing income from Form 4835 line 7 and Schedule K-1.",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_BASIS_REDUCTION",
        "Business {0} asset {1} investment credit basis reduction",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_DISPOSAL_ADDITIONAL_DEPRECIATION",
        "Business {0} asset {1} disposal additional depreciation",
    ),
    ("LINE_BUSINESS_N_ASSET_N_FORM3468_BASIS", "Business {0} asset {1} Form 3468 credit base"),
    (
        "LINE_BUSINESS_N_ASSET_N_FORM3468_BUILDING_BASIS",
        "Business {0} asset {1} Form 3468 building basis",
    ),
    ("LINE_BUSINESS_N_ASSET_N_FORM3468_CREDIT", "Business {0} asset {1} Form 3468 credit"),
    (
        "LINE_BUSINESS_N_ASSET_N_FORM3468_PERIOD_EXPENDITURES",
        "Business {0} asset {1} Form 3468 period expenditures",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_FORM4255_CARRYOVER",
        "Business {0} asset {1} Form 4255 carryforward cut",
    ),
    ("LINE_BUSINESS_N_ASSET_N_FORM4255_CLAIMED", "Business {0} asset {1} Form 4255 credit used"),
    (
        "LINE_BUSINESS_N_ASSET_N_FORM4255_FREED",
        "Business {0} asset {1} Form 4255 other credits freed",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_FORM4255_PERCENT",
        "Business {0} asset {1} Form 4255 recapture percent",
    ),
    ("LINE_BUSINESS_N_ASSET_N_FORM4255_RECAPTURE", "Business {0} asset {1} Form 4255 recapture"),
    ("LINE_BUSINESS_N_ASSET_N_FORM4255_TAX", "Business {0} asset {1} Form 4255 recapture tax"),
    ("LINE_BUSINESS_N_ASSET_N_FORM4255_YEARS", "Business {0} asset {1} Form 4255 years held"),
    ("LINE_FORM8582CR_LINE_2A", "Rehabilitation credit for 2026"),
    ("LINE_FORM8582CR_LINE_2B", "Rehabilitation credit from prior years"),
    ("LINE_FORM8582CR_LINE_2C", "Rehabilitation credit, total"),
    ("LINE_INVESTMENT_CREDIT_RECAPTURE_TAX", "Investment credit recapture tax"),
    (
        "WARN_ESTATE_INVESTMENT_INTEREST_LEFT_OUT",
        "This K-1 needs its actual regular/AMT investment-income or capital-loss portions before the Form 4952 source limit can be determined. The dependent deduction and election benefit is left out; ordinary income and tax remain included. Accepted borrowing and carryforward records are preserved.",
    ),
    (
        "WARN_ESTATE_QBI_SOURCE_LEFT_OUT",
        "QBI statement {0}: its current or released loss character cannot be attributed to the actual beneficiary activity from these source records. The affected QBI benefit is left out; ordinary income and tax remain included.",
    ),
    (
        "LINE_ESTATE_TRUST_N_INVESTMENT_N_BASIS_REDUCTION",
        "Estate or trust {0} investment {1} investment credit basis reduction",
    ),
    (
        "LINE_ESTATE_TRUST_N_INVESTMENT_N_FORM3468_BASIS",
        "Estate or trust {0} investment {1} Form 3468 credit base",
    ),
    (
        "LINE_ESTATE_TRUST_N_INVESTMENT_N_FORM3468_CREDIT",
        "Estate or trust {0} investment {1} Form 3468 credit",
    ),
    (
        "LINE_K1_ACTIVITY_N_INVESTMENT_N_BASIS_REDUCTION",
        "K-1 activity {0} investment {1} investment credit basis reduction",
    ),
    (
        "LINE_K1_ACTIVITY_N_INVESTMENT_N_FORM3468_BASIS",
        "K-1 activity {0} investment {1} Form 3468 credit base",
    ),
    (
        "LINE_K1_ACTIVITY_N_INVESTMENT_N_FORM3468_CREDIT",
        "K-1 activity {0} investment {1} Form 3468 credit",
    ),
    (
        "LINE_PARTNER_N_INVESTMENT_N_BASIS_REDUCTION",
        "Partner {0} investment {1} investment credit basis reduction",
    ),
    (
        "LINE_PARTNER_N_INVESTMENT_N_FORM3468_BASIS",
        "Partner {0} investment {1} Form 3468 credit base",
    ),
    ("LINE_PARTNER_N_INVESTMENT_N_FORM3468_CREDIT", "Partner {0} investment {1} Form 3468 credit"),
    (
        "LINE_RENTAL_N_BUILDING_N_BASIS_REDUCTION",
        "Rental {0} building {1} investment credit basis reduction",
    ),
    ("LINE_RENTAL_N_BUILDING_N_FORM3468_BASIS", "Rental {0} building {1} Form 3468 credit base"),
    (
        "LINE_RENTAL_N_BUILDING_N_FORM3468_BUILDING_BASIS",
        "Rental {0} building {1} Form 3468 building basis",
    ),
    ("LINE_RENTAL_N_BUILDING_N_FORM3468_CREDIT", "Rental {0} building {1} Form 3468 credit"),
    (
        "LINE_RENTAL_N_BUILDING_N_FORM3468_PERIOD_EXPENDITURES",
        "Rental {0} building {1} Form 3468 period expenditures",
    ),
    (
        "LINE_RENTAL_N_BUILDING_N_FORM4255_CARRYOVER",
        "Rental {0} building {1} Form 4255 carryforward cut",
    ),
    ("LINE_RENTAL_N_BUILDING_N_FORM4255_CLAIMED", "Rental {0} building {1} Form 4255 credit used"),
    (
        "LINE_RENTAL_N_BUILDING_N_FORM4255_FREED",
        "Rental {0} building {1} Form 4255 other credits freed",
    ),
    (
        "LINE_RENTAL_N_BUILDING_N_FORM4255_PERCENT",
        "Rental {0} building {1} Form 4255 recapture percent",
    ),
    ("LINE_RENTAL_N_BUILDING_N_FORM4255_RECAPTURE", "Rental {0} building {1} Form 4255 recapture"),
    ("LINE_RENTAL_N_BUILDING_N_FORM4255_TAX", "Rental {0} building {1} Form 4255 recapture tax"),
    ("LINE_RENTAL_N_BUILDING_N_FORM4255_YEARS", "Rental {0} building {1} Form 4255 years held"),
    ("LINE_RENTAL_N_SALE_BUILDING_N_RECAPTURE", "Rental {0} sale building section {1} recapture"),
    (
        "LINE_RENTAL_N_SALE_BUILDING_ADDITIONAL_DEPRECIATION",
        "Rental {0} sale building additional depreciation",
    ),
    (
        "LINE_SCORP_N_INVESTMENT_N_BASIS_REDUCTION",
        "S corporation {0} investment {1} investment credit basis reduction",
    ),
    (
        "LINE_SCORP_N_INVESTMENT_N_FORM3468_BASIS",
        "S corporation {0} investment {1} Form 3468 credit base",
    ),
    (
        "LINE_SCORP_N_INVESTMENT_N_FORM3468_CREDIT",
        "S corporation {0} investment {1} Form 3468 credit",
    ),
    (
        "EXPLAIN_FORM3468_BUILDING_BASIS",
        "Form 3468 line 1e: the building's adjusted basis when the substantial-rehabilitation measuring period began.",
    ),
    (
        "EXPLAIN_FORM3468_PERIOD_EXPENDITURES",
        "Form 3468 line 1f: the qualified rehabilitation expenditures during the measuring period, which must exceed both $5,000 and line 1e.",
    ),
    (
        "EXPLAIN_FORM3468_QUALIFIED_EXPENDITURES",
        "Form 3468 line 1g: the qualified rehabilitation expenditures of this building, the cost of this row (after a recapture, the part not recaptured).",
    ),
    (
        "EXPLAIN_FORM3468_CREDIT_BASE",
        "Form 3468: the credit base of this energy property, its business share of cost less any section 179 deduction (Part V line 1a or 3a; Part VI Section B, F, H or I).",
    ),
    (
        "EXPLAIN_FORM3468_REHABILITATION_CREDIT",
        "Form 3468 lines 1j and 3: 4% of line 1g, year {0} of the building's five-year credit period. It goes to Form 3800 Part III line 4k.",
    ),
    (
        "EXPLAIN_OWN_FORM3468_CREDIT",
        "Gross credit for this actual K-1 property from your completed Form 3468, before Form 3800 limits. It replaces the automatic credit and supplies the current outside-basis reduction once.",
    ),
    (
        "EXPLAIN_FORM3468_ENERGY_CREDIT",
        "Form 3468: the credit determined, {0}% of the credit base (the rate and bonuses of Part I lines 7 to 10). It goes to Form 3800 Part III line 4a (Part VI) or 1v (Part V).",
    ),
    (
        "EXPLAIN_FORM3468_EARLIER_ENERGY_CREDIT",
        "Form 3468: the energy credit determined in an earlier year for this property; it is not claimed again for 2026.",
    ),
    (
        "EXPLAIN_INVESTMENT_CREDIT_BASIS_REDUCTION",
        "Section 50(c): the amount that reduces this property's depreciable basis for the investment credit, the whole rehabilitation credit determined or half of an energy credit.",
    ),
    (
        "EXPLAIN_FORM4255_CLAIMED",
        "Form 4255 line B6: the credit from this property used on your earlier returns.",
    ),
    (
        "EXPLAIN_FORM4255_YEARS",
        "Form 4255 line D12: the whole years from the first day of the month the property was placed in service until it was disposed of or stopped being investment credit property.",
    ),
    (
        "EXPLAIN_FORM4255_FREED",
        "Form 4255 line D13: other credits you could have used instead (Worksheet 2).",
    ),
    (
        "EXPLAIN_FORM4255_PERCENT",
        "Form 4255 line D15: the percent of the credit recaptured for those years (100%, 80%, 60%, 40% or 20%; none after five years).",
    ),
    (
        "EXPLAIN_FORM4255_RECAPTURE",
        "Form 4255 line D16: the credit used less line D13, times the recapture percent.",
    ),
    (
        "EXPLAIN_FORM4255_CARRYOVER",
        "Form 4255 Part I column (i): the same percent of this property's credit still carried forward, which reduces that carryforward.",
    ),
    (
        "EXPLAIN_FORM4255_TAX",
        "Form 4255 Part I column (r): the recapture added to your tax on Schedule 2 line 13a. It is outside the Form 3800 limits.",
    ),
    (
        "EXPLAIN_ASSET_ADDITIONAL_DEPRECIATION",
        "Form 4797 line 28a: the depreciation allowed, including the section 50(c) reduction of an investment credit, over straight line on the cost before that reduction.",
    ),
    (
        "EXPLAIN_SCHEDULE2_LINE_13A_INVESTMENT_CREDIT",
        "Schedule 2 line 13a: the investment credit recapture of Form 4255 (identified 3468). It is added to the tax after the credits.",
    ),
    (
        "EXPLAIN_FORM8582CR_LINE_2A",
        "Form 8582-CR line 2a: 2026 rehabilitation credits of rental real estate activities (Worksheet 2, column (a)).",
    ),
    (
        "EXPLAIN_FORM8582CR_LINE_2B",
        "Form 8582-CR line 2b: prior-year unallowed rehabilitation credits of rental real estate activities (Worksheet 2, column (b)).",
    ),
    ("EXPLAIN_FORM8582CR_LINE_2C", "Form 8582-CR line 2c: lines 2a and 2b."),
    (
        "EXPLAIN_FORM8582CR_LINE_17",
        "Form 8582-CR line 17: line 7, the credits left after the tax on net passive income.",
    ),
    ("EXPLAIN_FORM8582CR_LINE_18", "Form 8582-CR line 18: line 16 (Part II is zero)."),
    ("EXPLAIN_FORM8582CR_LINE_19", "Form 8582-CR line 19: line 17 less line 18."),
    ("EXPLAIN_FORM8582CR_LINE_20", "Form 8582-CR line 20: the smaller of line 2c and line 19."),
    (
        "EXPLAIN_FORM8582CR_LINE_21",
        "Form 8582-CR line 21: $250,000 ($125,000 married filing separately and living apart).",
    ),
    ("EXPLAIN_FORM8582CR_LINE_22", "Form 8582-CR line 22: your modified adjusted gross income."),
    ("EXPLAIN_FORM8582CR_LINE_23", "Form 8582-CR line 23: line 21 less line 22, not below zero."),
    (
        "EXPLAIN_FORM8582CR_LINE_24",
        "Form 8582-CR line 24: half of line 23, at most $25,000 ($12,500 married filing separately and living apart).",
    ),
    (
        "EXPLAIN_FORM8582CR_LINE_25",
        "Form 8582-CR line 25: the rental losses the special allowance already let you deduct (Form 8582 line 9).",
    ),
    (
        "EXPLAIN_FORM8582CR_LINE_26",
        "Form 8582-CR line 26: line 24 less line 25, the special allowance that remains for rehabilitation credits.",
    ),
    (
        "EXPLAIN_FORM8582CR_LINE_27",
        "Form 8582-CR line 27: the regular tax on your taxable income less the tax on it reduced by line 26.",
    ),
    ("EXPLAIN_FORM8582CR_LINE_28", "Form 8582-CR line 28: line 18."),
    ("EXPLAIN_FORM8582CR_LINE_29", "Form 8582-CR line 29: line 27 less line 28."),
    ("EXPLAIN_FORM8582CR_LINE_30", "Form 8582-CR line 30: the smaller of line 20 and line 29."),
    ("EXPLAIN_FORM8582CR_LINE_32", "Form 8582-CR line 32: line 30."),
    (
        "WARN_REHABILITATION_CREDIT_LEFT_OUT",
        "{0}: a rehabilitation credit whose eligibility isn't confirmed isn't supported, so the 2026 credit is left out of this estimate; your tax may be lower. If you completed Form 3468 yourself, enter its credit in More credits → General business credits, line 4k, and enter the property with the basis reduction from that form.",
    ),
    (
        "WARN_ENERGY_CREDIT_LEFT_OUT",
        "{0}: an energy credit whose eligibility isn't confirmed isn't supported, so it is left out of this estimate and the property is depreciated on its full cost; your tax may be lower. If you completed Form 3468 yourself, enter its credit in More credits → General business credits, line 4a or 1v, and enter the property with the basis reduction from that form.",
    ),
    (
        "WARN_K1_HOUSING_CREDIT_LEFT_OUT",
        "{0}: a low-income housing credit from an interest acquired before 1990 has its own phase-out, which is not calculated, so its credit is left out of this estimate; your tax may be lower.",
    ),
    (
        "WARN_K1_INVESTMENT_CREDIT_LEFT_OUT",
        "{0}: investment credit property that isn't confirmed isn't supported, so its credit is left out of this estimate; your tax may be lower. If you completed Form 3468 yourself, enter its gross credit on this K-1's matching property row and answer its expense and basis reduction question.",
    ),
    (
        "REHABILITATION_PERIOD_AFTER_PLACED",
        "Rehabilitation credit: the measuring period must begin before the date the building was placed in service.",
    ),
    (
        "INVESTMENT_CREDIT_HISTORY_ABOVE_CREDIT",
        "The credit used and carried forward cannot exceed the credit determined for this property in that year.",
    ),
    (
        "K1_INVESTMENT_NEEDS_CONFIRMATION",
        "Answer whether this K-1's investment credit property is confirmed.",
    ),
    (
        "K1_HOUSING_NEEDS_CONFIRMATION",
        "Answer whether you acquired your interest in this K-1's low-income housing credit after 1989.",
    ),
    ("LINE_BUSINESS_CREDIT_CREDIT_N", "Business credit {0} {1}"),
    ("LINE_BUSINESS_CREDIT_CREDIT_N_CARRYBACK_N", "Business credit {0} {1} carryback {2}"),
    ("LINE_BUSINESS_CREDIT_CREDIT_N_EXPIRED", "Business credit {0} {1} expired"),
    ("LINE_BUSINESS_CREDIT_CREDIT_N_PASSIVE", "Business credit {0} {1} passive"),
    ("LINE_BUSINESS_CREDIT_CREDIT_N_PASSIVE_ALLOWED", "Business credit {0} {1} passive allowed"),
    ("LINE_BUSINESS_CREDIT_CREDIT_N_PASSIVE_TO_N", "Business credit {0} {1} passive to {2}"),
    ("LINE_BUSINESS_CREDIT_CREDIT_N_TO_N", "Business credit {0} {1} to {2}"),
    ("LINE_BUSINESS_CREDIT_CREDIT_N_USED", "Business credit {0} {1} used"),
    (
        "LINE_AMT_RENTAL_N_SALE_RELEASED_LOSS",
        "AMT rental {0} loss allowed in the ratio of this year's installment gain (section 469(g)(3))",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_DISPOSAL_INSTALLMENT_ORDINARY",
        "Business {0} asset {1} disposal Form 6252 line 26 to Form 4797 line 11",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_DISPOSAL_POSTPONED",
        "Business {0} asset {1} disposal gain postponed under section 1033",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_PARTIAL_DEPRECIATION",
        "Business {0} asset {1} partial disposition depreciation allowed on the part",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_PARTIAL_GAIN",
        "Business {0} asset {1} partial disposition gain or loss",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_SECTION179_PRIOR",
        "Business {0} asset {1} section 179 deduction taken (Form 4797 line 35(a))",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_SECTION179_RECAPTURE",
        "Business {0} asset {1} section 179 recapture as other income (line 37(a))",
    ),
    (
        "LINE_BUSINESS_N_ASSET_N_SECTION179_RECOMPUTED",
        "Business {0} asset {1} depreciation that would have been allowable on it (line 36(a))",
    ),
    (
        "LINE_BUSINESS_N_SALE_COVENANT",
        "Business {0} sale of the business, covenant not to compete (Schedule 1 line 8z)",
    ),
    (
        "LINE_BUSINESS_N_SALE_INVENTORY",
        "Business {0} sale of the business, inventory (Schedule C line 1)",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DISPOSAL_N_RECAPTURE",
        "Employee expenses {0} vehicle {1} disposal {2} recapture",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DISPOSAL_ADDITIONAL_DEPRECIATION",
        "Employee expenses {0} vehicle {1} disposal additional depreciation",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DISPOSAL_ADJUSTED_BASIS",
        "Employee expenses {0} vehicle {1} disposal adjusted basis",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DISPOSAL_COST",
        "Employee expenses {0} vehicle {1} disposal cost",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DISPOSAL_DEPRECIATION",
        "Employee expenses {0} vehicle {1} disposal depreciation",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DISPOSAL_FORM6252_ADJUSTED_BASIS",
        "Employee expenses {0} vehicle {1} disposal Form 6252 adjusted basis",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DISPOSAL_FORM6252_ASSUMED_DEBT",
        "Employee expenses {0} vehicle {1} disposal Form 6252 assumed debt",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DISPOSAL_FORM6252_CONTRACT_PRICE",
        "Employee expenses {0} vehicle {1} disposal Form 6252 contract price",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DISPOSAL_FORM6252_COST",
        "Employee expenses {0} vehicle {1} disposal Form 6252 cost",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DISPOSAL_FORM6252_DEEMED_PAYMENT",
        "Employee expenses {0} vehicle {1} disposal Form 6252 deemed payment",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DISPOSAL_FORM6252_DEPRECIATION",
        "Employee expenses {0} vehicle {1} disposal Form 6252 depreciation",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DISPOSAL_FORM6252_EXCESS_DEBT",
        "Employee expenses {0} vehicle {1} disposal Form 6252 excess debt",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DISPOSAL_FORM6252_EXPENSES",
        "Employee expenses {0} vehicle {1} disposal Form 6252 expenses",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DISPOSAL_FORM6252_GAIN",
        "Employee expenses {0} vehicle {1} disposal Form 6252 gain",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DISPOSAL_FORM6252_GROSS_PROFIT",
        "Employee expenses {0} vehicle {1} disposal Form 6252 gross profit",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DISPOSAL_FORM6252_PROFIT",
        "Employee expenses {0} vehicle {1} disposal Form 6252 profit before exclusion",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DISPOSAL_FORM6252_PERCENTAGE",
        "Employee expenses {0} vehicle {1} disposal Form 6252 gross profit percentage",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DISPOSAL_FORM6252_INCOME",
        "Employee expenses {0} vehicle {1} disposal Form 6252 income",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DISPOSAL_FORM6252_NET_PRICE",
        "Employee expenses {0} vehicle {1} disposal Form 6252 net price",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DISPOSAL_FORM6252_ORDINARY",
        "Employee expenses {0} vehicle {1} disposal Form 6252 ordinary",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DISPOSAL_FORM6252_PAYMENTS",
        "Employee expenses {0} vehicle {1} disposal Form 6252 payments",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DISPOSAL_FORM6252_PRICE",
        "Employee expenses {0} vehicle {1} disposal Form 6252 price",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DISPOSAL_FORM6252_RECAPTURE",
        "Employee expenses {0} vehicle {1} disposal Form 6252 recapture",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DISPOSAL_FORM6252_TOTAL",
        "Employee expenses {0} vehicle {1} disposal Form 6252 total",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DISPOSAL_FORM6252_TOTAL_PAYMENTS",
        "Employee expenses {0} vehicle {1} disposal Form 6252 total payments",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DISPOSAL_FORM6252_UNRECAPTURED_N",
        "Employee expenses {0} vehicle {1} disposal Form 6252 unrecaptured {2}",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DISPOSAL_GAIN",
        "Employee expenses {0} vehicle {1} disposal gain",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DISPOSAL_INSTALLMENT_GAIN",
        "Employee expenses {0} vehicle {1} disposal installment gain",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DISPOSAL_INSTALLMENT_ORDINARY",
        "Employee expenses {0} vehicle {1} disposal Form 6252 line 26 to Form 4797 line 11",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DISPOSAL_ORDINARY",
        "Employee expenses {0} vehicle {1} disposal ordinary",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DISPOSAL_POSTPONED",
        "Employee expenses {0} vehicle {1} disposal gain postponed under section 1033",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DISPOSAL_SALE_PRICE",
        "Employee expenses {0} vehicle {1} disposal sale price",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_DISPOSAL_SECTION1231",
        "Employee expenses {0} vehicle {1} disposal section 1231",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_LISTED_PRIOR_DEPRECIATION",
        "Employee expenses {0} vehicle {1} depreciation allowed in the earlier years (Form 4797 line 35(b))",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_LISTED_RECAPTURE",
        "Employee expenses {0} vehicle {1} recapture of excess depreciation (line 37(b))",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_LISTED_RECOMPUTED",
        "Employee expenses {0} vehicle {1} straight-line depreciation for the earlier years (line 36(b))",
    ),
    (
        "LINE_EMPLOYEE_EXPENSES_N_VEHICLE_N_OTHER_INCOME",
        "Employee expenses {0} vehicle {1} recapture of excess depreciation as other income (Schedule 1 line 8z)",
    ),
    (
        "LINE_FORM4797_LINE1B",
        "Form 4797 partial dispositions of MACRS assets, total gain (line 1b)",
    ),
    (
        "LINE_FORM4797_LINE1C",
        "Form 4797 partial dispositions of MACRS assets, total loss (line 1c)",
    ),
    (
        "LINE_INSTALLMENT_N_QUALIFIED_INCOME",
        "Installment sale {0} ordinary part that is qualified business income",
    ),
    (
        "LINE_PERSONAL_RENTAL_N_ASSET_N_DISPOSAL_N_RECAPTURE",
        "Rental equipment {0} asset {1} disposal {2} recapture",
    ),
    (
        "LINE_PERSONAL_RENTAL_N_ASSET_N_DISPOSAL_ADJUSTED_BASIS",
        "Rental equipment {0} asset {1} disposal adjusted basis",
    ),
    (
        "LINE_PERSONAL_RENTAL_N_ASSET_N_DISPOSAL_COST",
        "Rental equipment {0} asset {1} disposal cost",
    ),
    (
        "LINE_PERSONAL_RENTAL_N_ASSET_N_DISPOSAL_DEPRECIATION",
        "Rental equipment {0} asset {1} disposal depreciation allowed",
    ),
    (
        "LINE_PERSONAL_RENTAL_N_ASSET_N_DISPOSAL_GAIN",
        "Rental equipment {0} asset {1} disposal gain",
    ),
    (
        "LINE_PERSONAL_RENTAL_N_ASSET_N_DISPOSAL_SALE_PRICE",
        "Rental equipment {0} asset {1} disposal sale price",
    ),
    (
        "LINE_RENTAL_N_BUILDING_N_PARTIAL_DEPRECIATION",
        "Rental {0} building {1} partial disposition depreciation allowed on the part",
    ),
    (
        "LINE_RENTAL_N_BUILDING_N_PARTIAL_GAIN",
        "Rental {0} building {1} partial disposition gain or loss",
    ),
    (
        "LINE_RENTAL_N_SALE_BUILDING_ORDINARY",
        "Rental {0} sale building gain, ordinary under section 1239 (Form 4797 line 11)",
    ),
    ("LINE_RENTAL_N_SALE_POSTPONED", "Rental {0} sale gain postponed under section 1033"),
    (
        "LINE_RENTAL_N_SALE_RELEASED_LOSS",
        "Rental {0} loss allowed in the ratio of this year's installment gain (section 469(g)(3))",
    ),
    (
        "LINE_RENTAL_N_SALE_ROW_N_ORDINARY",
        "Rental {0} sale of building row {1} held one year or less, ordinary (Form 4797 line 11)",
    ),
    (
        "EXPLAIN_0RENTAL_N_SALE_RELEASED_LOSS",
        "The part of the rental's overall loss (this year's loss and the unallowed balance) that is allowed this year because the installment sale of the whole property recognizes this share of its gain (section 469(g)(3), Form 8582 instructions, Disposition of an Entire Interest). The rest stays unallowed until later payments recognize more gain.",
    ),
    (
        "EXPLAIN_ASSET_DISPOSAL_POSTPONED",
        "Gain from the condemnation that is not taxed this year because the cost of the replacement property is at least the amount received for it (section 1033); it lowers the replacement property's basis.",
    ),
    (
        "EXPLAIN_ASSET_SECTION179_PRIOR",
        "Form 4797 Part IV line 35(a): the section 179 deduction taken on property that is now used 50% or less for business or was kept for personal use.",
    ),
    (
        "EXPLAIN_ASSET_SECTION179_RECOMPUTED",
        "Form 4797 Part IV line 36(a): the depreciation that would have been allowable on the section 179 amount from the year the property was placed in service through this year.",
    ),
    (
        "EXPLAIN_ASSET_SECTION179_RECAPTURE",
        "Form 4797 Part IV line 37(a): the section 179 deduction above the depreciation that would have been allowable is other income of the business, and the property's basis rises by it (section 179(d)(10)).",
    ),
    (
        "EXPLAIN_BUSINESS_N_SALE_COVENANT",
        "A payment for your covenant not to compete in the sale of the business is ordinary income on Schedule 1 line 8z, not self-employment income.",
    ),
    (
        "EXPLAIN_BUSINESS_N_SALE_INVENTORY",
        "The price of the inventory sold with the business is ordinary business income counted with your sales (Schedule C line 1, Pub. 544).",
    ),
    (
        "EXPLAIN_EMPLOYEE_EXPENSES_N_VEHICLE_N_OTHER_INCOME",
        "The excess depreciation on a Form 2106 vehicle whose business use fell to 50% or less is other income on Schedule 1 line 8z, the schedule the deduction reached (Form 4797 Part IV).",
    ),
    (
        "EXPLAIN_INSTALLMENT_N_QUALIFIED_INCOME",
        "The part of this earlier-year installment gain that your section 1231 lookback makes ordinary is qualified business income of the business that sold the property.",
    ),
    (
        "EXPLAIN_N_FORM4797_LINE1B",
        "The gain on partial dispositions of MACRS assets that lines 2, 11 and 26 include (Form 4797 line 1b).",
    ),
    (
        "EXPLAIN_N_FORM4797_LINE1C",
        "The loss on partial dispositions of MACRS assets that lines 2 and 11 include (Form 4797 line 1c).",
    ),
    (
        "EXPLAIN_N_INSTALLMENT_ORDINARY",
        "Ordinary gain from an installment sale of property held one year or less: Form 6252 line 26 goes to Form 4797 line 11, marked From Form 6252.",
    ),
    (
        "EXPLAIN_N_ORDINARY_RELATED",
        "Gain on depreciable property sold to a corporation, partnership, trust or estate you control is ordinary income (section 1239), Form 4797 Part II line 11.",
    ),
    (
        "EXPLAIN_N_PARTIAL_DEPRECIATION",
        "The depreciation allowed on the replaced or removed part: its share of the building's depreciation through the month of the disposition (Regulations section 1.168(i)-8(f)(3)).",
    ),
    (
        "EXPLAIN_N_PARTIAL_GAIN",
        "Gain or loss on the replaced or removed part of a building: the amount received less the part's cost and plus its depreciation allowed (Form 4797 line 2, or line 11 when held one year or less; Regulations section 1.168(i)-8).",
    ),
    (
        "EXPLAIN_RENTAL_SALE_POSTPONED",
        "Gain from the condemnation that is not taxed this year because the cost of the replacement property is at least the amount received for it (section 1033).",
    ),
    (
        "EXPLAIN_RENTAL_SALE_ROW_ORDINARY",
        "Gain or loss on a building or improvement held one year or less is ordinary (Form 4797 Part II line 11), because section 1231 needs more than a year.",
    ),
    (
        "INSTALLMENT_RELATED_BUYER",
        "An installment sale to a relative or to a company you control is not calculated.",
    ),
    (
        "NOTE_SALE_TO_RELATIVE_LOSS_DISALLOWED",
        "{0}: a loss on a sale to a relative or to a company you control is not deductible, so it is left out.",
    ),
    (
        "PARTIAL_DISPOSITION_BEFORE_SERVICE",
        "The part must be replaced or removed after the building was placed in service.",
    ),
    (
        "PARTIAL_DISPOSITION_PART_ABOVE_COST",
        "The cost of the part must be less than the building's cost.",
    ),
    (
        "WARN_BUSINESS_SALE_LEFT_OUT",
        "{0}: the sale of purchased goodwill or another section 197 intangible, or an allocation the buyer reports differently, isn't supported, so the sale-of-the-business amounts are left out of this estimate; your tax may be higher.",
    ),
    ("ROW_LABEL", "{0} {1} — {2}"),
    (
        "WARN_K1_ITEM_LEFT_OUT",
        "{0}: {1} isn't supported, so it's left out of this estimate; your tax may be lower.",
    ),
    ("K1_ITEM_DEBT_EXCLUSION", "a section 108 exclusion of canceled debt (code E)"),
    (
        "K1_ITEM_RECOVERY_EXCLUSION",
        "an exclusion for a code J recovery that gave no earlier tax benefit",
    ),
    (
        "K1_ITEM_CRP_PAYMENTS",
        "excluding box 20 code AQ conservation reserve program payments from self-employment tax",
    ),
    (
        "K1_ITEM_PARTNER_DEDUCTION_PROPERTY_TWENTY",
        "box 13 code F capital gain property contributions (20%)",
    ),
    ("K1_ITEM_PARTNER_DEDUCTION_CONTRIBUTIONS_HUNDRED", "box 13 code G contributions (100%)"),
    ("K1_ITEM_PARTNER_DEDUCTION_SECTION59E", "box 13 code J section 59(e)(2) expenditures"),
    (
        "K1_ITEM_PARTNER_DEDUCTION_PORTFOLIO_OTHER",
        "box 13 code L deductions — portfolio income (other)",
    ),
    ("K1_ITEM_PARTNER_DEDUCTION_EDUCATIONAL", "box 13 code N educational assistance benefits"),
    ("K1_ITEM_PARTNER_DEDUCTION_PREPRODUCTIVE", "box 13 code P preproductive period expenses"),
    ("K1_ITEM_PARTNER_DEDUCTION_REFORESTATION", "box 13 code S reforestation expense deduction"),
    (
        "K1_ITEM_PARTNER_DEDUCTION_SOIL_WATER",
        "box 13 code W soil and water conservation expenditures",
    ),
    ("K1_ITEM_PARTNER_DEDUCTION_PRODUCTION", "box 13 code X production expenses"),
    ("K1_ITEM_PARTNER_DEDUCTION_BARRIERS", "box 13 code Y expenditures for removal of barriers"),
    ("K1_ITEM_PARTNER_DEDUCTION_ITEMIZED", "box 13 code Z itemized deductions"),
    (
        "K1_ITEM_PARTNER_DEDUCTION_CONSTRUCTION_FUND",
        "box 13 code AA contributions to a capital construction fund",
    ),
    (
        "K1_ITEM_PARTNER_DEDUCTION_DISTRIBUTION_INTEREST",
        "box 13 code AC interest on debt-financed distributions",
    ),
    (
        "K1_ITEM_PARTNER_DEDUCTION_WORKING_INTEREST",
        "box 13 code AD interest on an oil or gas working interest",
    ),
    ("K1_ITEM_PARTNER_DEDUCTION_OTHER", "box 13 code ZZ other deductions"),
    (
        "K1_ITEM_SCORP_DEDUCTION_PROPERTY_TWENTY",
        "box 12 code F capital gain property contributions (20%)",
    ),
    ("K1_ITEM_SCORP_DEDUCTION_CONTRIBUTIONS_HUNDRED", "box 12 code G contributions (100%)"),
    ("K1_ITEM_SCORP_DEDUCTION_SECTION59E", "box 12 code J section 59(e)(2) expenditures"),
    (
        "K1_ITEM_SCORP_DEDUCTION_PORTFOLIO_OTHER",
        "box 12 code L deductions — portfolio income (other)",
    ),
    ("K1_ITEM_SCORP_DEDUCTION_PREPRODUCTIVE", "box 12 code M preproductive period expenses"),
    ("K1_ITEM_SCORP_DEDUCTION_REFORESTATION", "box 12 code O reforestation expense deduction"),
    (
        "K1_ITEM_SCORP_DEDUCTION_SOIL_WATER",
        "box 12 code W soil and water conservation expenditures",
    ),
    ("K1_ITEM_SCORP_DEDUCTION_PRODUCTION", "box 12 code X production expenditures"),
    ("K1_ITEM_SCORP_DEDUCTION_BARRIERS", "box 12 code Y expenditures for removal of barriers"),
    ("K1_ITEM_SCORP_DEDUCTION_ITEMIZED", "box 12 code Z itemized deductions"),
    (
        "K1_ITEM_SCORP_DEDUCTION_CONSTRUCTION_FUND",
        "box 12 code AA contributions to a capital construction fund",
    ),
    (
        "K1_ITEM_SCORP_DEDUCTION_DISTRIBUTION_INTEREST",
        "box 12 code AC interest on debt-financed distributions",
    ),
    ("K1_ITEM_SCORP_DEDUCTION_OTHER", "box 12 code ZZ other deductions"),
    (
        "EXPLAIN_K1_ITEM_PORTFOLIO",
        "K-1 code A other portfolio income, Schedule 1 line 8z; net investment income (Form 8960 line 7) and Form 4952 investment income, never passive income or QBI.",
    ),
    (
        "EXPLAIN_K1_ITEM_RECOVERY",
        "K-1 code J recovery of a tax benefit item, Schedule 1 line 8z in full; an exclusion for a recovery that gave no earlier benefit is not applied.",
    ),
    (
        "EXPLAIN_K1_ITEM_DEBT",
        "K-1 box 11 code E canceled debt, Schedule 1 line 8c in full; a section 108 exclusion is not applied. It is income of the K-1's activity.",
    ),
    ("EXPLAIN_K1_ITEM_WINNINGS", "K-1 code K gambling winnings, Schedule 1 line 8b."),
    (
        "EXPLAIN_K1_ITEM_GAMBLING_LOSSES",
        "K-1 code K gambling losses that the basis and at-risk limits allow, Schedule A line 17a with your own: 90 percent of them, and only to the extent of winnings.",
    ),
    (
        "EXPLAIN_K1_ITEM_SECTION1256",
        "K-1 code C section 1256 contracts, one Form 6781 Part I line 1 row: 40 percent short-term and 60 percent long-term.",
    ),
    (
        "EXPLAIN_K1_ITEM_PREFERRED",
        "K-1 box 11 code Q Fannie Mae or Freddie Mac preferred stock, Schedule D line 5 or 12 by how long it was held.",
    ),
    (
        "EXPLAIN_K1_ITEM_ORDINARY",
        "K-1 box 11 code R specially allocated ordinary gain (loss), Form 4797 Part II line 11 and Schedule 1 line 4; an item of the K-1's activity.",
    ),
    (
        "EXPLAIN_K1_ITEM_NONPORTFOLIO",
        "K-1 code S non-portfolio capital gain (loss), Schedule D line 5 or 12; an item of the K-1's activity.",
    ),
    (
        "EXPLAIN_K1_ITEM_CHARITY_THIRTY",
        "K-1 code B or D contribution (30% limit), Schedule A line 11 or 12 in the section 170(b)(1)(B) class with your own gifts.",
    ),
    (
        "EXPLAIN_K1_ITEM_CHARITY_FIFTY",
        "K-1 code C noncash contribution (50% limit), Schedule A line 12 in the section 170(b)(1)(A) class with your own gifts.",
    ),
    (
        "EXPLAIN_K1_ITEM_CHARITY_PROPERTY",
        "K-1 code E capital gain property to a 50% organization (30% limit), Schedule A line 12 in the section 170(b)(1)(C) class.",
    ),
    (
        "EXPLAIN_K1_ITEM_INVESTMENT_INTEREST",
        "K-1 code H investment interest expense, Form 4952 line 1 with your own.",
    ),
    (
        "EXPLAIN_K1_ROYALTY_DEDUCTIONS",
        "K-1 code I royalty expenses allowed after the basis and at-risk limits, Schedule E line 19. These portfolio expenses do not enter Form 8582.",
    ),
    (
        "EXPLAIN_K1_ROYALTIES",
        "Royalties from Schedule K-1: gross receipts on Schedule E line 4, allowed code I expenses on lines 19 and 20, and the difference on line 21. Portfolio royalties are outside the passive-loss calculation, QBI and self-employment income; their net amount remains investment income.",
    ),
    (
        "EXPLAIN_K1_ITEM_EARLY_PENALTY",
        "K-1 code AB penalty on early withdrawal of savings, Schedule 1 line 18.",
    ),
    (
        "EXPLAIN_K1_DISTRIBUTION_GAIN",
        "K-1 box 19 cash above your basis: section 731(a)(1) gain on Form 8949 box C or F by how long you held the interest.",
    ),
    (
        "EXPLAIN_K1_SALE_ORDINARY",
        "K-1 box 20 code AB section 751 gain on the sale of your interest, Form 4797 Part II line 11 and Schedule 1 line 4.",
    ),
    (
        "EXPLAIN_K1_SALE_GAIN",
        "The sale or cash liquidation of your whole interest, Form 8949 box C or F: proceeds {0,number,currency} after the section 751 part, basis {1,number,currency}. Codes AC and AD are parts of a long-term gain.",
    ),
    (
        "EXPLAIN_K1_ORDINARY_GAINS",
        "Ordinary gains from Schedules K-1 (box 11 code R and the section 751 gain on a sale of an interest): Form 4797 Part II line 11, then Schedule 1 line 4.",
    ),
    (
        "EXPLAIN_K1_DEFERRED_COMPENSATION_TAX",
        "K-1 box 20 code AI interest and additional tax on deferred compensation, Schedule 2 line 13h.",
    ),
    (
        "EXPLAIN_FORM8582_PART_IX_N_ITEMS_UNALLOWED",
        "Form 8582 Part IX column (d): the part of this activity's other-income losses (code R and code S) that the passive activity rules do not allow.",
    ),
    (
        "EXPLAIN_FORM8582_PART_IX_N_ITEMS_ALLOWED",
        "Form 8582 Part IX column (e): the part of this activity's other-income losses (code R and code S) that the passive activity rules allow.",
    ),
    ("LINE_PARTNERSHIP_N_LINE4_ROYALTIES", "Partnership {0} Schedule E line 4 royalties"),
    (
        "LINE_PARTNERSHIP_N_LINE19_ROYALTY_DEDUCTIONS",
        "Partnership {0} Schedule E line 19 royalty deductions",
    ),
    (
        "LINE_PARTNERSHIP_N_LINE20_ROYALTY_EXPENSES",
        "Partnership {0} Schedule E line 20 royalty expenses",
    ),
    (
        "LINE_PARTNERSHIP_N_LINE21_ROYALTY_INCOME",
        "Partnership {0} Schedule E line 21 royalty income or loss",
    ),
    ("LINE_SCORP_N_LINE4_ROYALTIES", "S corporation {0} Schedule E line 4 royalties"),
    (
        "LINE_SCORP_N_LINE19_ROYALTY_DEDUCTIONS",
        "S corporation {0} Schedule E line 19 royalty deductions",
    ),
    (
        "LINE_SCORP_N_LINE20_ROYALTY_EXPENSES",
        "S corporation {0} Schedule E line 20 royalty expenses",
    ),
    (
        "LINE_SCORP_N_LINE21_ROYALTY_INCOME",
        "S corporation {0} Schedule E line 21 royalty income or loss",
    ),
    ("LINE_AMT_PARTNERSHIP_N_LINE4_ROYALTIES", "AMT partnership {0} Schedule E line 4 royalties"),
    (
        "LINE_AMT_PARTNERSHIP_N_LINE19_ROYALTY_DEDUCTIONS",
        "AMT partnership {0} Schedule E line 19 royalty deductions",
    ),
    (
        "LINE_AMT_PARTNERSHIP_N_LINE20_ROYALTY_EXPENSES",
        "AMT partnership {0} Schedule E line 20 royalty expenses",
    ),
    (
        "LINE_AMT_PARTNERSHIP_N_LINE21_ROYALTY_INCOME",
        "AMT partnership {0} Schedule E line 21 royalty income or loss",
    ),
    ("LINE_AMT_SCORP_N_LINE4_ROYALTIES", "AMT S corporation {0} Schedule E line 4 royalties"),
    (
        "LINE_AMT_SCORP_N_LINE19_ROYALTY_DEDUCTIONS",
        "AMT S corporation {0} Schedule E line 19 royalty deductions",
    ),
    (
        "LINE_AMT_SCORP_N_LINE20_ROYALTY_EXPENSES",
        "AMT S corporation {0} Schedule E line 20 royalty expenses",
    ),
    (
        "LINE_AMT_SCORP_N_LINE21_ROYALTY_INCOME",
        "AMT S corporation {0} Schedule E line 21 royalty income or loss",
    ),
    ("LINE_PARTNERSHIP_N_ITEM_N", "Partnership {0} box {1}"),
    ("LINE_SCORP_N_ITEM_N", "S corporation {0} box {1}"),
    ("LINE_PARTNERSHIP_N_DISTRIBUTION_GAIN", "Partnership {0} cash distribution above basis"),
    ("LINE_PARTNERSHIP_N_SALE_ORDINARY", "Partnership {0} section 751 gain on the sale"),
    ("LINE_PARTNERSHIP_N_SALE_GAIN", "Partnership {0} capital gain on the sale"),
    ("LINE_SCORP_N_SALE_GAIN", "S corporation {0} capital gain on the sale of shares"),
    ("LINE_K1_ORDINARY_GAINS", "Ordinary gains from Schedules K-1"),
    ("LINE_K1_DEFERRED_COMPENSATION_TAX", "Tax on deferred compensation from Schedules K-1"),
    (
        "LINE_FORM8582_PART_IX_N_ITEMS_UNALLOWED",
        "Form 8582 Part IX activity {0}: other-income losses, unallowed",
    ),
    (
        "LINE_FORM8582_PART_IX_N_ITEMS_ALLOWED",
        "Form 8582 Part IX activity {0}: other-income losses, allowed",
    ),
    ("DISPLAY_K1P_OTHER_INCOME", "Other income (box 11)"),
    ("DISPLAY_K1P_SALE", "Sale of your interest"),
    ("DISPLAY_K1S_OTHER_INCOME", "Other income (box 10)"),
    ("DISPLAY_INT_OWNER_PREMIUM", "Premium, accrued interest and discount"),
    ("DISPLAY_INT_OWNER_INCOME", "Interest, exempt interest and foreign tax"),
    ("DISPLAY_DIV_OWNER_MAIN", "Dividends and distributions"),
    ("DISPLAY_DIV_OWNER_OTHER", "Special parts, exempt and foreign tax"),
    ("DISPLAY_SALE_B_CORRECTIONS", "Corrections and state boxes"),
    ("DISPLAY_R1099_BOXES_1_7", "Boxes 1–7"),
    ("DISPLAY_R1099_BOXES_8_19", "Boxes 8–19"),
    ("DISPLAY_R1099_AFTER_DEATH", "After death and state"),
    ("DISPLAY_OFFICE_AREAS", "Areas and home"),
    ("DISPLAY_OFFICE_HOME_COSTS", "Whole-home costs"),
    ("DISPLAY_OFFICE_ONLY_COSTS", "Office-only costs and carryovers"),
    ("DISPLAY_K1S_BOXES_1_5", "Corporation and boxes 1–5b"),
    ("DISPLAY_K1S_BOX_16_17", "Distributions and QBI"),
    ("DISPLAY_K1P_ALLOCATION_STATEMENTS", "Income and property statements"),
    ("DISPLAY_K1P_BOXES_1_14", "Boxes 1–4 and 14"),
    ("DISPLAY_K1P_BOXES_19_20", "Distributions, QBI and other boxes"),
    ("DISPLAY_K1P_K3_BASIS", "Schedule K-3, basis and disposition"),
    ("DISPLAY_DOCUMENT_BASIS_RECORDS", "Basis, AMT and disposition records"),
    ("DISPLAY_FTC_INCOME", "Foreign income and taxes"),
    ("DISPLAY_PERSON_IDENTITY", "Identity"),
    ("DISPLAY_8986_PART_I", "Part I Information about entity submitting this form"),
    (
        "DISPLAY_8986_PART_II",
        "Part II Information about the audited partnership or partnership that filed an AAR",
    ),
    ("DISPLAY_8986_PART_III", "Part III Information about the pass-through partner"),
    ("DISPLAY_8986_PART_IV", "Part IV Information about the partner receiving this statement"),
    (
        "DISPLAY_8986_PART_V",
        "Part V Partner's total reviewed year income, gain, loss, deduction, credits and other items",
    ),
    ("DISPLAY_8986_PART_VI", "Part VI Statements"),
    ("DISPLAY_MARKETPLACE_RESIDENCE", "Residence and forms"),
    (
        "NOTE_QTP_ROTH_NOT_QUALIFIED",
        "{0}: the 529 transfer is not a qualified rollover, so its earnings are taxed as a withdrawal; enter the amount deposited as a regular Roth contribution on the IRA contributions page.",
    ),
    (
        "EXPLAIN_ROTH_N_N_FIRST_HOME",
        "Form 8606 line 20: a qualified first-time homebuyer distribution from a Roth IRA whose first contribution year is 2021 or earlier, up to $10,000 in a lifetime. It is taken out before the contributions and conversions of line 22 and line 24, and Form 5329 shows it on lines 1 and 2 with exception 09.",
    ),
    ("LINE_ROTH_N_FIRST_HOME", "Roth {0} first-home distribution (Form 8606 line 20)"),
    (
        "LINE_ROTH_TAXPAYER_DESIGNATED",
        "Roth designated disaster share, taxpayer (Form 8606 line 25b)",
    ),
    ("LINE_ROTH_SPOUSE_DESIGNATED", "Roth designated disaster share, spouse (Form 8606 line 25b)"),
    (
        "EXPLAIN_ROTH_N_DESIGNATED",
        "Form 8606 line 25b: line 25a times the qualified disaster distributions over line 21. It is reported on Form 8915-F line 19 and spread over three years, and it is not in Form 8606 line 25c.",
    ),
    (
        "LINE_IRA_EXCESS_TAXPAYER_LINE12_RETURNED_PRIOR_EXCESS",
        "IRA excess line 12 returned prior excess, taxpayer",
    ),
    (
        "LINE_IRA_EXCESS_SPOUSE_LINE12_RETURNED_PRIOR_EXCESS",
        "IRA excess line 12 returned prior excess, spouse",
    ),
    // Wage specials (2026 W-2 and Form 1040).
    ("LINE_ADDITIONAL_MEDICARE_TOTAL_WITHHELD", "Additional Medicare Tax withheld"),
    (
        "W2_RAILROAD_WITH_SOCIAL_SECURITY_BOXES",
        "A railroad W-2 lists compensation and taxes in box 14a; boxes 3 through 7 must be blank.",
    ),
    (
        "W2_TA_ABOVE_LIMIT",
        "Employer Trump account contributions reported with code TA cannot exceed $2,500. Ask the employer to correct the form.",
    ),
    (
        "ISSUE_SEPARATE_SPOUSE_BOXES_WITH_SPOUSE_ITEMIZING",
        "Spouse standard deduction boxes require a spouse who is not filing; they cannot be combined with a spouse who itemizes.",
    ),
    (
        "WARN_EMPLOYEE_WAGES_PAYROLL_TAX_LEFT_OUT",
        "{0}: the pay remains income on Form 1040 line 1h; its payroll tax is not calculated, so tax may be higher.",
    ),
    (
        "EXPLAIN_1040_1B",
        "Household employee pay without a W-2 is Form 1040 line 1b, with no self-employment tax.",
    ),
    (
        "EXPLAIN_1040_1D",
        "Medicaid waiver payments outside W-2 box 1 are Form 1040 line 1d; excluded payments also subtract on Schedule 1 line 8s.",
    ),
    (
        "EXPLAIN_1040_1I",
        "The App compares each person’s combat-pay and Medicaid earned-income choices and keeps the largest earned income credit plus additional child tax credit. This election does not add taxable income.",
    ),
    (
        "EXPLAIN_MEDICAID_WAIVER_EXCLUDED",
        "Notice 2014-7 payments excluded from income subtract on Schedule 1 line 8s. They may still count for credits and nondeductible IRA room.",
    ),
    (
        "EXPLAIN_STRIKE_BENEFITS",
        "Strike or lockout benefits other than true gifts are earned income on Form 1040 line 1h.",
    ),
    (
        "EXPLAIN_EMPLOYEE_WAGES_NOT_COVERED",
        "Employee pay outside ordinary Social Security and Medicare coverage remains on line 1h; payroll tax is omitted with a notice.",
    ),
    (
        "EXPLAIN_8959_WAGES_AND_RRTA",
        "Form 8959 applies a separate full filing-status threshold to Medicare wages and RRTA compensation. Line 19 includes ordinary W-2 box 12 codes B/N with box 6, but excludes RRTA B/N. Ordinary Medicare and RRTA compensation have separate full thresholds; only the reconciled Additional Medicare withholding is a payment.",
    ),
    (
        "EXPLAIN_8812_COMBAT_PAY",
        "Schedule 8812 always counts all nontaxable combat pay, independently of the earned income credit election.",
    ),
    (
        "EXPLAIN_BUSINESS_STATUTORY_W2",
        "Statutory employee box 1 is Schedule C gross receipts and qualified business income after expenses, without self-employment tax. Earned income uses gross receipts.",
    ),
    ("LINE_ADDITIONAL_MEDICARE_ORDINARY_WITHHELD", "Box 6 plus ordinary uncollected Medicare tax"),
    ("LINE_ADDITIONAL_MEDICARE_WITHHOLDING_WAGES", "Medicare wages for withholding reconciliation"),
    ("LINE_ADDITIONAL_MEDICARE_REGULAR_WITHHOLDING", "Regular Medicare withholding"),
    (
        "LINE_ADDITIONAL_MEDICARE_ORDINARY_ADDITIONAL_WITHHELD",
        "Additional Medicare withholding on wages",
    ),
    ("LINE_ADDITIONAL_MEDICARE_SE_INCOME", "Self-employment income"),
    ("LINE_ADDITIONAL_MEDICARE_SE_THRESHOLD", "Self-employment Additional Medicare threshold"),
    (
        "LINE_ADDITIONAL_MEDICARE_SE_WAGE_OFFSET",
        "Medicare wages reducing the self-employment threshold",
    ),
    (
        "LINE_ADDITIONAL_MEDICARE_SE_REMAINING_THRESHOLD",
        "Remaining self-employment Additional Medicare threshold",
    ),
    ("LINE_ADDITIONAL_MEDICARE_SE_EXCESS", "Self-employment income above the remaining threshold"),
    ("LINE_ADDITIONAL_MEDICARE_SE_TAX", "Additional Medicare tax on self-employment income"),
    ("DISPLAY_W2_PAYROLL", "Box 14a railroad retirement"),
    ("LINE_HOUSEHOLD_WAGES", "Household wages"),
    ("LINE_MEDICAID_WAIVER_WAGES", "Medicaid waiver wages"),
    ("LINE_STRIKE_BENEFITS", "Strike benefits"),
    ("LINE_EMPLOYEE_WAGES_NOT_COVERED", "Employee wages not covered"),
    ("LINE_MEDICAID_WAIVER_EXCLUDED", "Medicaid waiver excluded"),
    ("LINE_COMBAT_PAY_ELECTION", "Combat pay election"),
    ("LINE_CHILD_CREDIT_COMBAT_PAY", "Child credit combat pay"),
    ("LINE_ADDITIONAL_MEDICARE_W2_WAGES", "Additional medicare w2 wages"),
    ("LINE_ADDITIONAL_MEDICARE_TIP_WAGES", "Additional medicare tip wages"),
    ("LINE_ADDITIONAL_MEDICARE_EMPLOYEE_WAGES", "Additional medicare employee wages"),
    ("LINE_ADDITIONAL_MEDICARE_WAGES_TOTAL", "Additional medicare wages total"),
    ("LINE_ADDITIONAL_MEDICARE_WAGES_THRESHOLD", "Additional medicare wages threshold"),
    ("LINE_ADDITIONAL_MEDICARE_WAGES_EXCESS", "Additional medicare wages excess"),
    ("LINE_ADDITIONAL_MEDICARE_PAYROLL_WAGES", "Additional medicare payroll wages"),
    ("LINE_ADDITIONAL_MEDICARE_RRTA_COMPENSATION", "Known RRTA compensation subtotal"),
    ("LINE_ADDITIONAL_MEDICARE_RRTA_THRESHOLD", "Additional medicare rrta threshold"),
    ("LINE_ADDITIONAL_MEDICARE_RRTA_EXCESS", "Additional medicare rrta excess"),
    ("LINE_ADDITIONAL_MEDICARE_RRTA_TAX", "Additional medicare rrta tax"),
    ("LINE_ADDITIONAL_MEDICARE_WAGES_AND_RRTA", "Additional medicare wages and rrta"),
    (
        "LINE_ADDITIONAL_MEDICARE_RRTA_WITHHELD",
        "Known RRTA Additional Medicare withholding subtotal",
    ),
    ("LINE_STATUTORY_WAGE_N_GROSS_RECEIPTS", "Statutory wage {0} gross receipts"),
    ("LINE_STATUTORY_WAGE_N_NET_PROFIT", "Statutory wage {0} net profit"),
    (
        "EXPLAIN_FORM8958_SE_TAX_TOTAL",
        "{0} — Form 8958 line 10: the actual partner's self-employment tax, attributed from the annual calculation across that person's sources.",
    ),
    (
        "EXPLAIN_FORM8958_SE_TAX_THIS_RETURN",
        "{0} — Form 8958 line 10: the actual partner reports the self-employment tax; the nonpartner spouse reports none.",
    ),
    (
        "EXPLAIN_FORM8958_SE_DEDUCTION_TOTAL",
        "{0} — Form 8958 line 9: the deductible part of the actual partner's self-employment tax, attributed from the annual calculation.",
    ),
    (
        "EXPLAIN_FORM8958_SE_DEDUCTION_THIS_RETURN",
        "{0} — Form 8958 line 9: the actual partner reports this deduction; the nonpartner spouse reports none.",
    ),
    ("LINE_FORM8958_SE_TAX_N_TOTAL", "Form 8958 partnership {0}, total self-employment tax"),
    (
        "LINE_FORM8958_SE_TAX_N_THIS_RETURN",
        "Form 8958 partnership {0}, this return's self-employment tax",
    ),
    (
        "LINE_FORM8958_SE_DEDUCTION_N_TOTAL",
        "Form 8958 partnership {0}, total deductible self-employment tax",
    ),
    (
        "LINE_FORM8958_SE_DEDUCTION_N_THIS_RETURN",
        "Form 8958 partnership {0}, this return's deductible self-employment tax",
    ),
    (
        "EXPLAIN_FORM8958_INTEREST_TOTAL",
        "{0} — Form 8958 line 2, interest: the whole source amount before community allocation.",
    ),
    ("LINE_FORM8958_INTEREST_N_TOTAL", "Form 8958 interest {0}, the whole amount"),
    (
        "EXPLAIN_FORM8958_INTEREST_THIS_RETURN",
        "{0} — Form 8958 line 2, interest: this return's allocated source amount.",
    ),
    ("LINE_FORM8958_INTEREST_N_THIS_RETURN", "Form 8958 interest {0}, this return's share"),
    (
        "EXPLAIN_FORM8958_DIVIDENDS_TOTAL",
        "{0} — Form 8958 line 3, dividends: the whole source amount before community allocation.",
    ),
    ("LINE_FORM8958_DIVIDENDS_N_TOTAL", "Form 8958 dividends {0}, the whole amount"),
    (
        "EXPLAIN_FORM8958_DIVIDENDS_THIS_RETURN",
        "{0} — Form 8958 line 3, dividends: this return's allocated source amount.",
    ),
    ("LINE_FORM8958_DIVIDENDS_N_THIS_RETURN", "Form 8958 dividends {0}, this return's share"),
    (
        "EXPLAIN_FORM8958_ROYALTIES_TOTAL",
        "{0} — Form 8958 line 8, royalties: the whole source amount before community allocation.",
    ),
    ("LINE_FORM8958_ROYALTIES_N_TOTAL", "Form 8958 royalties {0}, the whole amount"),
    (
        "EXPLAIN_FORM8958_ROYALTIES_THIS_RETURN",
        "{0} — Form 8958 line 8, royalties: this return's allocated source amount.",
    ),
    ("LINE_FORM8958_ROYALTIES_N_THIS_RETURN", "Form 8958 royalties {0}, this return's share"),
    ("EXPLAIN_FORM8958_GAIN_N_TOTAL", "{0} — Form 8958 line 6: the whole amount."),
    ("LINE_FORM8958_GAIN_N_TOTAL", "Form 8958 gain {0}, the whole amount"),
    ("EXPLAIN_FORM8958_GAIN_N_THIS_RETURN", "{0} — Form 8958 line 6: this return's share."),
    (
        "EXPLAIN_FORM8958_GAIN_ORDINARY",
        "{0} — ordinary property gain included in this return's Form 8958 share.",
    ),
    (
        "EXPLAIN_FORM8958_GAIN_SECTION1231",
        "{0} — section 1231 property gain or loss included in this return's Form 8958 share.",
    ),
    ("LINE_FORM8958_GAIN_N_THIS_RETURN", "Form 8958 gain {0}, this return's share"),
    ("LINE_FORM8958_GAIN_N_ORDINARY_THIS_RETURN", "Form 8958 gain {0}, ordinary property part"),
    (
        "LINE_FORM8958_GAIN_N_SECTION1231_THIS_RETURN",
        "Form 8958 gain {0}, section 1231 property part",
    ),
    ("EXPLAIN_FORM8958_CREDIT_N_TOTAL", "{0} — Form 8958 line 12: the whole amount."),
    ("LINE_FORM8958_CREDIT_N_TOTAL", "Form 8958 credit {0}, the whole amount"),
    ("EXPLAIN_FORM8958_CREDIT_N_THIS_RETURN", "{0} — Form 8958 line 12: this return's share."),
    ("LINE_FORM8958_CREDIT_N_THIS_RETURN", "Form 8958 credit {0}, this return's share"),
    ("LINE_FORM_N_TAXPAYER_N_PRINT_N", "Form {0}-F {1} disasters, taxpayer: {2}"),
    (
        "LINE_FORM_N_TAXPAYER_N_WORKSHEET_PRIOR_N",
        "{0}-F {1} disasters, taxpayer: Qualified distributions in earlier years for FEMA DR-{2}",
    ),
    (
        "LINE_FORM_N_TAXPAYER_N_WORKSHEET_LIMIT_N",
        "{0}-F {1} disasters, taxpayer: Remaining qualified distribution limit for FEMA DR-{2}",
    ),
    (
        "LINE_FORM_N_TAXPAYER_N_WORKSHEET_QUALIFIED_N",
        "{0}-F {1} disasters, taxpayer: Qualified distributions this year for FEMA DR-{2}",
    ),
    ("LINE_FORM_N_SPOUSE_N_PRINT_N", "Form {0}-F {1} disasters, spouse: {2}"),
    (
        "LINE_FORM_N_SPOUSE_N_WORKSHEET_PRIOR_N",
        "{0}-F {1} disasters, spouse: Qualified distributions in earlier years for FEMA DR-{2}",
    ),
    (
        "LINE_FORM_N_SPOUSE_N_WORKSHEET_LIMIT_N",
        "{0}-F {1} disasters, spouse: Remaining qualified distribution limit for FEMA DR-{2}",
    ),
    (
        "LINE_FORM_N_SPOUSE_N_WORKSHEET_QUALIFIED_N",
        "{0}-F {1} disasters, spouse: Qualified distributions this year for FEMA DR-{2}",
    ),
    ("LINE_SCHEDULE_D_CHOICE_N_YES", "Schedule D line {0} — Yes"),
    ("LINE_SCHEDULE_D_CHOICE_N_NO", "Schedule D line {0} — No"),
    (
        "EXPLAIN_SCHEDULE_D_SUMMARY_ANSWER",
        "The summary uses your short-term and long-term results, qualified dividends and the investment-interest form included in this review.",
    ),
    ("LINE_CAPITAL_GAIN_1A_UNROUNDED", "Capital gain 1a before rounding"),
    (
        "LINE_CAPITAL_GAIN_1A_BASIS_UNROUNDED",
        "Capital gain 1a cost or other basis (e) before rounding",
    ),
    ("LINE_CAPITAL_GAIN_1A_PROCEEDS_UNROUNDED", "Capital gain 1a proceeds (d) before rounding"),
    ("LINE_CAPITAL_GAIN_1B_UNROUNDED", "Capital gain 1b before rounding"),
    (
        "LINE_CAPITAL_GAIN_1B_ADJUSTMENT_UNROUNDED",
        "Capital gain 1b adjustments (g) before rounding",
    ),
    (
        "LINE_CAPITAL_GAIN_1B_BASIS_UNROUNDED",
        "Capital gain 1b cost or other basis (e) before rounding",
    ),
    ("LINE_CAPITAL_GAIN_1B_PROCEEDS_UNROUNDED", "Capital gain 1b proceeds (d) before rounding"),
    ("LINE_CAPITAL_GAIN_8A_UNROUNDED", "Capital gain 8a before rounding"),
    (
        "LINE_CAPITAL_GAIN_8A_BASIS_UNROUNDED",
        "Capital gain 8a cost or other basis (e) before rounding",
    ),
    ("LINE_CAPITAL_GAIN_8A_PROCEEDS_UNROUNDED", "Capital gain 8a proceeds (d) before rounding"),
    ("LINE_CAPITAL_GAIN_8B_UNROUNDED", "Capital gain 8b before rounding"),
    (
        "LINE_CAPITAL_GAIN_8B_ADJUSTMENT_UNROUNDED",
        "Capital gain 8b adjustments (g) before rounding",
    ),
    (
        "LINE_CAPITAL_GAIN_8B_BASIS_UNROUNDED",
        "Capital gain 8b cost or other basis (e) before rounding",
    ),
    ("LINE_CAPITAL_GAIN_8B_PROCEEDS_UNROUNDED", "Capital gain 8b proceeds (d) before rounding"),
    ("LINE_CAPITAL_GAIN_N_UNROUNDED", "Capital gain {0} before rounding"),
    (
        "LINE_CAPITAL_GAIN_N_ADJUSTMENT_UNROUNDED",
        "Capital gain {0} adjustments (g) before rounding",
    ),
    (
        "LINE_CAPITAL_GAIN_N_BASIS_UNROUNDED",
        "Capital gain {0} cost or other basis (e) before rounding",
    ),
    ("LINE_CAPITAL_GAIN_N_PROCEEDS_UNROUNDED", "Capital gain {0} proceeds (d) before rounding"),
    ("LINE_QBI_COLUMN_N_LINE_N", "QBI column {0}, line {1}"),
    ("LINE_QBI_COLUMNS_TOTAL", "Total qualified business income components"),
    ("LINE_QBI_LOSS_COLUMN_N_ADJUSTED", "QBI activity {0}, income after loss netting"),
    ("LINE_QBI_LOSS_COLUMN_N_INCOME", "QBI activity {0}, income before loss netting"),
    ("LINE_QBI_LOSS_COLUMN_N_REDUCTION", "QBI activity {0}, allocated loss reduction"),
    ("LINE_QBI_LOSS_LINE_N", "QBI loss netting, line {0}"),
    ("LINE_QBI_AGGREGATION_N_MEMBER_N_INCOME", "QBI aggregation {0}, member {1} income"),
    ("LINE_QBI_AGGREGATION_N_MEMBER_N_WAGES", "QBI aggregation {0}, member {1} W-2 wages"),
    (
        "LINE_QBI_AGGREGATION_N_MEMBER_N_PROPERTY",
        "QBI aggregation {0}, member {1} qualified property",
    ),
    ("LINE_QBI_AGGREGATION_N_TOTAL_INCOME", "QBI aggregation {0}, combined income"),
    ("LINE_QBI_AGGREGATION_N_TOTAL_WAGES", "QBI aggregation {0}, combined W-2 wages"),
    ("LINE_QBI_AGGREGATION_N_TOTAL_PROPERTY", "QBI aggregation {0}, combined qualified property"),
    (
        "EXPLAIN_QBI_COMPUTED_COLUMN",
        "This form line uses the settled qualified business income, allowed losses, wages, property and applicable phase-in from this calculation.",
    ),
    (
        "EXPLAIN_QBI_AGGREGATION_SOURCE",
        "Actual member income, W-2 wages and qualified property enter the elected aggregation before its combined wage/property limit and cross-business loss netting.",
    ),
    ("LINE_SCHEDULE_D_LINE_N", "Schedule D line {0}"),
    ("LINE_SCHEDULE_D_LINE_N_UNROUNDED", "Schedule D line {0} before rounding"),
    (
        "EXPLAIN_SCHEDULE_D_COMPUTED_TOTAL",
        "Schedule D uses the same admitted capital sources and prior losses as your estimate. K-1 gains, capital gain distributions and carryovers remain separate; the summary keeps short-term and long-term character before the annual loss limit.",
    ),
    (
        "EXPLAIN_FORM8915_PROJECTED_LINE",
        "Form 8915-F review from the same accepted distributions, basis, repayments and timing used in your estimate. Each filer and disaster beginning year has a separate form.",
    ),
    (
        "EXPLAIN_FORM8915_DISASTER_LIMIT",
        "The actual allocations to this disaster share one $22,000 limit across plans and years. Earlier qualified distributions reduce the amount available this year; Worksheet 1B uses these same per-disaster facts when applicable.",
    ),
    ("LINE_FORM_N_TAXPAYER_N_CHOICE_N_YES", "Form {0}-F {1} disasters, taxpayer: {2} — Yes"),
    ("LINE_FORM_N_TAXPAYER_N_CHOICE_N_NO", "Form {0}-F {1} disasters, taxpayer: {2} — No"),
    ("LINE_FORM_N_SPOUSE_N_CHOICE_N_YES", "Form {0}-F {1} disasters, spouse: {2} — Yes"),
    ("LINE_FORM_N_SPOUSE_N_CHOICE_N_NO", "Form {0}-F {1} disasters, spouse: {2} — No"),
    ("LINE_FORM_N_TAXPAYER_N_PRINT_1A", "Form {0}-F {1} disasters, taxpayer: line 1a"),
    ("LINE_FORM_N_TAXPAYER_N_PRINT_1B", "Form {0}-F {1} disasters, taxpayer: line 1b"),
    ("LINE_FORM_N_TAXPAYER_N_PRINT_1C", "Form {0}-F {1} disasters, taxpayer: line 1c"),
    ("LINE_FORM_N_TAXPAYER_N_PRINT_1D", "Form {0}-F {1} disasters, taxpayer: line 1d"),
    ("LINE_FORM_N_TAXPAYER_N_PRINT_1E", "Form {0}-F {1} disasters, taxpayer: line 1e"),
    ("LINE_FORM_N_TAXPAYER_N_PRINT_5A", "Form {0}-F {1} disasters, taxpayer: line 5a"),
    ("LINE_FORM_N_TAXPAYER_N_PRINT_5B", "Form {0}-F {1} disasters, taxpayer: line 5b"),
    ("LINE_FORM_N_SPOUSE_N_PRINT_1A", "Form {0}-F {1} disasters, spouse: line 1a"),
    ("LINE_FORM_N_SPOUSE_N_PRINT_1B", "Form {0}-F {1} disasters, spouse: line 1b"),
    ("LINE_FORM_N_SPOUSE_N_PRINT_1C", "Form {0}-F {1} disasters, spouse: line 1c"),
    ("LINE_FORM_N_SPOUSE_N_PRINT_1D", "Form {0}-F {1} disasters, spouse: line 1d"),
    ("LINE_FORM_N_SPOUSE_N_PRINT_1E", "Form {0}-F {1} disasters, spouse: line 1e"),
    ("LINE_FORM_N_SPOUSE_N_PRINT_5A", "Form {0}-F {1} disasters, spouse: line 5a"),
    ("LINE_FORM_N_SPOUSE_N_PRINT_5B", "Form {0}-F {1} disasters, spouse: line 5b"),
    (
        "LINE_FORM_N_TAXPAYER_N_HOME_PLAN_N_GROSS",
        "Form {0}-F {1} disasters, taxpayer: employer plan document {2} home distribution gross",
    ),
    (
        "LINE_FORM_N_TAXPAYER_N_HOME_PLAN_N_COST",
        "Form {0}-F {1} disasters, taxpayer: employer plan document {2} home distribution cost",
    ),
    (
        "LINE_FORM_N_TAXPAYER_N_HOME_PLAN_N_REPAID",
        "Form {0}-F {1} disasters, taxpayer: employer plan document {2} home distribution repaid",
    ),
    (
        "LINE_FORM_N_TAXPAYER_N_HOME_IRA_N_GROSS",
        "Form {0}-F {1} disasters, taxpayer: traditional IRA document {2} home distribution gross",
    ),
    (
        "LINE_FORM_N_TAXPAYER_N_HOME_IRA_N_COST",
        "Form {0}-F {1} disasters, taxpayer: traditional IRA document {2} home distribution cost",
    ),
    (
        "LINE_FORM_N_TAXPAYER_N_HOME_IRA_N_REPAID",
        "Form {0}-F {1} disasters, taxpayer: traditional IRA document {2} home distribution repaid",
    ),
    (
        "LINE_FORM_N_TAXPAYER_N_HOME_ROTH_N_GROSS",
        "Form {0}-F {1} disasters, taxpayer: Roth IRA document {2} home distribution gross",
    ),
    (
        "LINE_FORM_N_TAXPAYER_N_HOME_ROTH_N_COST",
        "Form {0}-F {1} disasters, taxpayer: Roth IRA document {2} home distribution cost",
    ),
    (
        "LINE_FORM_N_TAXPAYER_N_HOME_ROTH_N_REPAID",
        "Form {0}-F {1} disasters, taxpayer: Roth IRA document {2} home distribution repaid",
    ),
    (
        "LINE_FORM_N_SPOUSE_N_HOME_PLAN_N_GROSS",
        "Form {0}-F {1} disasters, spouse: employer plan document {2} home distribution gross",
    ),
    (
        "LINE_FORM_N_SPOUSE_N_HOME_PLAN_N_COST",
        "Form {0}-F {1} disasters, spouse: employer plan document {2} home distribution cost",
    ),
    (
        "LINE_FORM_N_SPOUSE_N_HOME_PLAN_N_REPAID",
        "Form {0}-F {1} disasters, spouse: employer plan document {2} home distribution repaid",
    ),
    (
        "LINE_FORM_N_SPOUSE_N_HOME_IRA_N_GROSS",
        "Form {0}-F {1} disasters, spouse: traditional IRA document {2} home distribution gross",
    ),
    (
        "LINE_FORM_N_SPOUSE_N_HOME_IRA_N_COST",
        "Form {0}-F {1} disasters, spouse: traditional IRA document {2} home distribution cost",
    ),
    (
        "LINE_FORM_N_SPOUSE_N_HOME_IRA_N_REPAID",
        "Form {0}-F {1} disasters, spouse: traditional IRA document {2} home distribution repaid",
    ),
    (
        "LINE_FORM_N_SPOUSE_N_HOME_ROTH_N_GROSS",
        "Form {0}-F {1} disasters, spouse: Roth IRA document {2} home distribution gross",
    ),
    (
        "LINE_FORM_N_SPOUSE_N_HOME_ROTH_N_COST",
        "Form {0}-F {1} disasters, spouse: Roth IRA document {2} home distribution cost",
    ),
    (
        "LINE_FORM_N_SPOUSE_N_HOME_ROTH_N_REPAID",
        "Form {0}-F {1} disasters, spouse: Roth IRA document {2} home distribution repaid",
    ),
    (
        "LINE_FORM_N_TAXPAYER_N_TERMINAL_PLAN",
        "Form {0}-F {1} disasters, taxpayer: remaining employer-plan installment on the final return",
    ),
    (
        "LINE_FORM_N_TAXPAYER_N_TERMINAL_IRA",
        "Form {0}-F {1} disasters, taxpayer: remaining IRA installment on the final return",
    ),
    (
        "LINE_FORM_N_SPOUSE_N_TERMINAL_PLAN",
        "Form {0}-F {1} disasters, spouse: remaining employer-plan installment on the final return",
    ),
    (
        "LINE_FORM_N_SPOUSE_N_TERMINAL_IRA",
        "Form {0}-F {1} disasters, spouse: remaining IRA installment on the final return",
    ),
    (
        "EXPLAIN_FORM8915_TERMINAL_REMAINDER",
        "The remaining future installment cannot spread beyond the year of death. It is included once in line 13 or 24, in addition to the current and prior-form installments.",
    ),
    (
        "LINE_FORM_N_TAXPAYER_N_CHOICE_N_WORKSHEET",
        "Form {0}-F {1} disasters, taxpayer: line {2} uses the per-disaster Worksheet 1B limits",
    ),
    (
        "LINE_FORM_N_SPOUSE_N_CHOICE_N_WORKSHEET",
        "Form {0}-F {1} disasters, spouse: line {2} uses the per-disaster Worksheet 1B limits",
    ),
    (
        "LINE_FORM_N_TAXPAYER_N_ELECTION_N_ALL",
        "Form {0}-F {1} disasters, taxpayer: line {2} — All taxable amounts elected for 2026",
    ),
    (
        "LINE_FORM_N_TAXPAYER_N_ELECTION_N_SPREAD",
        "Form {0}-F {1} disasters, taxpayer: line {2} — Three-year spread elected",
    ),
    (
        "LINE_FORM_N_SPOUSE_N_ELECTION_N_ALL",
        "Form {0}-F {1} disasters, spouse: line {2} — All taxable amounts elected for 2026",
    ),
    (
        "LINE_FORM_N_SPOUSE_N_ELECTION_N_SPREAD",
        "Form {0}-F {1} disasters, spouse: line {2} — Three-year spread elected",
    ),
    (
        "EXPLAIN_FORM8915_ELECTION",
        "Your timing election applies to every qualified distribution of this disaster year on this filer's form. A final return includes the remaining income in the year of death.",
    ),
    // Relative-year calculation IDs retain the original 2026-build labels.
    ("LINE_ARCHER_N_PRIOR_EXCESS_DEDUCTED_RETURN_YEAR", "Archer {0} prior excess deducted 2026"),
    (
        "LINE_ARCHER_N_PRIOR_EXCESS_REMAINING_FOLLOWING_YEAR",
        "Archer {0} prior excess remaining 2027",
    ),
    ("LINE_ARCHER_N_RETURN_YEAR_EXCESS_ACCOUNT_VALUE", "Archer {0} 2026 excess account value"),
    (
        "LINE_ARCHER_N_RETURN_YEAR_UNCORRECTED_EMPLOYER_EXCESS",
        "Archer {0} 2026 uncorrected employer excess",
    ),
    (
        "LINE_ARCHER_N_RETURN_YEAR_UNCORRECTED_EMPLOYER_EXCESS_OTHER_INCOME",
        "Archer {0} 2026 uncorrected employer excess other income",
    ),
    (
        "LINE_ARCHER_N_RETURN_YEAR_UNCORRECTED_PERSONAL_EXCESS",
        "Archer {0} 2026 uncorrected personal excess",
    ),
    ("LINE_ARCHER_PRIOR_EXCESS_DEDUCTED_RETURN_YEAR", "Archer prior excess deducted 2026"),
    ("LINE_ARCHER_PRIOR_EXCESS_REMAINING_FOLLOWING_YEAR", "Archer prior excess remaining 2027"),
    ("LINE_ARCHER_RETURN_YEAR_EXCESS_ACCOUNT_VALUE", "Archer 2026 excess account value"),
    ("LINE_ARCHER_RETURN_YEAR_EXCESS_CONTRIBUTION_TAX", "Archer 2026 excess contribution tax"),
    (
        "LINE_ARCHER_RETURN_YEAR_UNCORRECTED_EMPLOYER_EXCESS",
        "Archer 2026 uncorrected employer excess",
    ),
    (
        "LINE_ARCHER_RETURN_YEAR_UNCORRECTED_EMPLOYER_EXCESS_OTHER_INCOME",
        "Archer 2026 uncorrected employer excess other income",
    ),
    (
        "LINE_ARCHER_RETURN_YEAR_UNCORRECTED_PERSONAL_EXCESS",
        "Archer 2026 uncorrected personal excess",
    ),
    (
        "LINE_CHARITY_CAPITAL_GAIN_THIRTY_RETURN_YEAR_CARRYFORWARD",
        "30% appreciated public-charity property: 2026 excess and eligible floor carryforward",
    ),
    (
        "LINE_CHARITY_CAPITAL_GAIN_TWENTY_RETURN_YEAR_CARRYFORWARD",
        "20% appreciated other-organization property: 2026 excess and eligible floor carryforward",
    ),
    ("LINE_CHARITY_CARRY_EXPIRED_AFTER_RETURN_YEAR", "Charitable carryovers expired after 2026"),
    (
        "LINE_CHARITY_CASH_SIXTY_RETURN_YEAR_CARRYFORWARD",
        "60% public-charity cash: 2026 excess and eligible floor carryforward",
    ),
    (
        "LINE_CHARITY_CONSERVATION_FIFTY_RETURN_YEAR_CARRYFORWARD",
        "50% conservation contributions: 2026 excess and eligible floor carryforward",
    ),
    (
        "LINE_CHARITY_CONSERVATION_HUNDRED_RETURN_YEAR_CARRYFORWARD",
        "100% qualified-farmer conservation contributions: 2026 excess and eligible floor carryforward",
    ),
    (
        "LINE_CHARITY_CURRENT_CASH_RETURN_YEAR_CARRYFORWARD",
        "Charity current cash 2026 carryforward",
    ),
    (
        "LINE_CHARITY_DISASTER_RELIEF_RETURN_YEAR_CARRYFORWARD",
        "qualified 2021 disaster-relief gifts: 2026 excess and eligible floor carryforward",
    ),
    (
        "LINE_CHARITY_HOSTED_STUDENT_RETURN_YEAR_CARRYFORWARD",
        "Charity hosted student 2026 carryforward",
    ),
    ("LINE_CHARITY_NONCASH_RETURN_YEAR_CARRYFORWARD", "Charity noncash 2026 carryforward"),
    (
        "LINE_CHARITY_OTHER_N_PERCENT_RETURN_YEAR_CARRYFORWARD",
        "Charity other {0} percent 2026 carryforward",
    ),
    (
        "LINE_CHARITY_OTHER_THIRTY_RETURN_YEAR_CARRYFORWARD",
        "30% other-organization gifts: 2026 excess and eligible floor carryforward",
    ),
    (
        "LINE_CHARITY_PRIOR_CAPITAL_GAIN_THIRTY_EXPIRED_AFTER_RETURN_YEAR",
        "30% appreciated public-charity property: balance expired after 2026",
    ),
    (
        "LINE_CHARITY_PRIOR_CAPITAL_GAIN_THIRTY_REMAINING_FOLLOWING_YEAR",
        "30% appreciated public-charity property: balance carried to 2027",
    ),
    (
        "LINE_CHARITY_PRIOR_CAPITAL_GAIN_TWENTY_EXPIRED_AFTER_RETURN_YEAR",
        "20% appreciated other-organization property: balance expired after 2026",
    ),
    (
        "LINE_CHARITY_PRIOR_CAPITAL_GAIN_TWENTY_REMAINING_FOLLOWING_YEAR",
        "20% appreciated other-organization property: balance carried to 2027",
    ),
    ("LINE_CHARITY_PRIOR_CASH_EXPIRED_AFTER_RETURN_YEAR", "Charity prior cash expired after 2026"),
    ("LINE_CHARITY_PRIOR_CASH_REMAINING_FOLLOWING_YEAR", "Charity prior cash remaining 2027"),
    (
        "LINE_CHARITY_PRIOR_CASH_SIXTY_EXPIRED_AFTER_RETURN_YEAR",
        "60% public-charity cash: balance expired after 2026",
    ),
    (
        "LINE_CHARITY_PRIOR_CASH_SIXTY_REMAINING_FOLLOWING_YEAR",
        "60% public-charity cash: balance carried to 2027",
    ),
    (
        "LINE_CHARITY_PRIOR_CONSERVATION_FIFTY_EXPIRED_AFTER_RETURN_YEAR",
        "50% conservation contributions: balance expired after 2026",
    ),
    (
        "LINE_CHARITY_PRIOR_CONSERVATION_FIFTY_REMAINING_FOLLOWING_YEAR",
        "50% conservation contributions: balance carried to 2027",
    ),
    (
        "LINE_CHARITY_PRIOR_CONSERVATION_HUNDRED_EXPIRED_AFTER_RETURN_YEAR",
        "100% qualified-farmer conservation contributions: balance expired after 2026",
    ),
    (
        "LINE_CHARITY_PRIOR_CONSERVATION_HUNDRED_REMAINING_FOLLOWING_YEAR",
        "100% qualified-farmer conservation contributions: balance carried to 2027",
    ),
    (
        "LINE_CHARITY_PRIOR_DISASTER_RELIEF_EXPIRED_AFTER_RETURN_YEAR",
        "qualified 2021 disaster-relief gifts: balance expired after 2026",
    ),
    (
        "LINE_CHARITY_PRIOR_DISASTER_RELIEF_REMAINING_FOLLOWING_YEAR",
        "qualified 2021 disaster-relief gifts: balance carried to 2027",
    ),
    (
        "LINE_CHARITY_PRIOR_HOUSEHOLD_N_REMAINING_FOLLOWING_YEAR",
        "Charity prior household {0} remaining 2027",
    ),
    (
        "LINE_CHARITY_PRIOR_OTHER_THIRTY_EXPIRED_AFTER_RETURN_YEAR",
        "30% other-organization gifts: balance expired after 2026",
    ),
    (
        "LINE_CHARITY_PRIOR_OTHER_THIRTY_REMAINING_FOLLOWING_YEAR",
        "30% other-organization gifts: balance carried to 2027",
    ),
    (
        "LINE_CHARITY_PRIOR_PROPERTY_FIFTY_EXPIRED_AFTER_RETURN_YEAR",
        "50% public-charity property: balance expired after 2026",
    ),
    (
        "LINE_CHARITY_PRIOR_PROPERTY_FIFTY_REMAINING_FOLLOWING_YEAR",
        "50% public-charity property: balance carried to 2027",
    ),
    (
        "LINE_CHARITY_PRIOR_STOCK_N_REMAINING_FOLLOWING_YEAR",
        "Charity prior stock {0} remaining 2027",
    ),
    (
        "LINE_CHARITY_PROPERTY_FIFTY_RETURN_YEAR_CARRYFORWARD",
        "50% public-charity property: 2026 excess and eligible floor carryforward",
    ),
    (
        "LINE_CHARITY_PUBLIC_STOCK_RETURN_YEAR_CARRYFORWARD",
        "Charity public stock 2026 carryforward",
    ),
    ("LINE_CHARITY_RETURN_YEAR_CARRYFORWARD", "Charity 2026 carryforward"),
    ("LINE_CHARITY_RETURN_YEAR_NOL_REDUCTION", "Charity 2026 NOL reduction"),
    ("LINE_CHARITY_WHALING_RETURN_YEAR_CARRYFORWARD", "Charity whaling 2026 carryforward"),
    (
        "LINE_FOREIGN_BRANCH_AMT_CARRYBACK_PRIOR_YEAR",
        "Schedule B (Form 1116) AMT foreign branch category: line 7, carryback to 2025",
    ),
    (
        "LINE_FOREIGN_BRANCH_AMT_DEDUCTION_YEAR_ABSORBED_PRIOR_YEAR",
        "Schedule B (Form 1116) AMT foreign branch category: line 7, absorbed in deduction year 2025",
    ),
    (
        "LINE_FOREIGN_BRANCH_REGULAR_CARRYBACK_PRIOR_YEAR",
        "Schedule B (Form 1116) foreign branch category: line 7, carryback to 2025",
    ),
    (
        "LINE_FOREIGN_BRANCH_REGULAR_DEDUCTION_YEAR_ABSORBED_PRIOR_YEAR",
        "Schedule B (Form 1116) foreign branch category: line 7, absorbed in deduction year 2025",
    ),
    ("LINE_FOREIGN_GENERAL_AMT_CARRYBACK_PRIOR_YEAR", "Foreign general AMT carryback 2025"),
    (
        "LINE_FOREIGN_GENERAL_AMT_DEDUCTION_YEAR_ABSORBED_PRIOR_YEAR",
        "Foreign general AMT deduction year absorbed 2025",
    ),
    ("LINE_FOREIGN_GENERAL_REGULAR_CARRYBACK_PRIOR_YEAR", "Foreign general regular carryback 2025"),
    (
        "LINE_FOREIGN_GENERAL_REGULAR_DEDUCTION_YEAR_ABSORBED_PRIOR_YEAR",
        "Foreign general regular deduction year absorbed 2025",
    ),
    ("LINE_FOREIGN_PASSIVE_AMT_CARRYBACK_PRIOR_YEAR", "Foreign passive AMT carryback 2025"),
    (
        "LINE_FOREIGN_PASSIVE_AMT_DEDUCTION_YEAR_ABSORBED_PRIOR_YEAR",
        "Foreign passive AMT deduction year absorbed 2025",
    ),
    ("LINE_FOREIGN_PASSIVE_REGULAR_CARRYBACK_PRIOR_YEAR", "Foreign passive regular carryback 2025"),
    (
        "LINE_FOREIGN_PASSIVE_REGULAR_DEDUCTION_YEAR_ABSORBED_PRIOR_YEAR",
        "Foreign passive regular deduction year absorbed 2025",
    ),
    ("LINE_HSA_N_PRIOR_EXCESS_DEDUCTED_RETURN_YEAR", "HSA {0} prior excess deducted 2026"),
    ("LINE_HSA_N_PRIOR_EXCESS_REMAINING_FOLLOWING_YEAR", "HSA {0} prior excess remaining 2027"),
    ("LINE_HSA_N_RETURN_YEAR_EXCESS_ACCOUNT_VALUE", "HSA {0} 2026 excess account value"),
    (
        "LINE_HSA_N_RETURN_YEAR_UNCORRECTED_EMPLOYER_EXCESS",
        "HSA {0} 2026 uncorrected employer excess",
    ),
    ("LINE_HSA_N_RETURN_YEAR_UNCORRECTED_EXCESS", "HSA {0} 2026 uncorrected excess"),
    ("LINE_HSA_RETURN_YEAR_EXCESS_CONTRIBUTION_TAX", "HSA 2026 excess contribution tax"),
    (
        "LINE_HSA_RETURN_YEAR_UNCORRECTED_EMPLOYER_EXCESS_OTHER_INCOME",
        "HSA 2026 uncorrected employer excess other income",
    ),
    ("LINE_REPAYMENT_RETURN_YEAR_DECREASE", "Repayment 2026 decrease"),
    (
        "LINE_REPAYMENT_RETURN_YEAR_ORIGINAL_CHAPTER_ONE_NET_TAX",
        "Repayment 2026 original chapter one net tax",
    ),
    (
        "LINE_REPAYMENT_RETURN_YEAR_REFIGURED_CHAPTER_ONE_NET_TAX",
        "Repayment 2026 refigured chapter one net tax",
    ),
    (
        "LINE_RMD_N_DEFERRED_TO_FOLLOWING_YEAR",
        "RMD {0} deferred to 2027 (required minimum distribution)",
    ),
];
