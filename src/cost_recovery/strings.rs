pub(crate) const COST_RECOVERY_STRINGS: &[(&str, &str)] = &[
    (
        "ISSUE_RENTAL_ENERGY_HISTORICAL_BASIS_UNAVAILABLE",
        "The earlier building deduction needs its actual cost-source association and opening basis reduction before depreciation can be calculated. Current rental income and cost records were kept.",
    ),
    (
        "RENTAL_ENERGY_CLAIM_RECORDS",
        "Select this rental's actual physical building and reconcile the unique project cost, construction and installation dates with its cost row.",
    ),
    (
        "RENTAL_ENERGY_CERTIFICATION_RECORDS",
        "Review the actual certification, license, reference standard, savings and increased-amount records.",
    ),
    (
        "RENTAL_ENERGY_BUILDING_ASSOCIATION",
        "The energy claim needs its actual saved physical building and cost-row identities. Correct the source selection in Rentals.",
    ),
    (
        "ISSUE_RENTAL_ENERGY_HISTORY_UNAVAILABLE",
        "no section 179D deduction was included because actual all-party building deduction records for the current and previous three years are unavailable. Known rental income and costs remain.",
    ),
    (
        "ISSUE_RENTAL_ENERGY_CERTIFICATION_UNAVAILABLE",
        "no section 179D deduction was included without an established qualifying unrelated professional certification. Known rental income and costs remain.",
    ),
    (
        "ISSUE_RENTAL_ENERGY_RETROFIT_BASIS_UNAVAILABLE",
        "the elected retrofit needs its final-certification-year adjusted-basis calculation. No retrofit deduction was included; known rental income and costs remain.",
    ),
    (
        "ISSUE_RENTAL_ENERGY_YEAR_UNAVAILABLE",
        "section 179D calculation parameters for this actual return year are unavailable. No new energy deduction was included.",
    ),
    (
        "ISSUE_RENTAL_ENERGY_BASIS_UNAVAILABLE",
        "no current section 179D benefit was included because this claim needs the original cost, service-year and disposition basis fit. Correct or supply those source records; known rental income remains.",
    ),
    (
        "ISSUE_RENTAL_ENERGY_BUILDING_ALLOCATION_UNAVAILABLE",
        "the same physical building has different documented savings or allowance rates. Its current section 179D allocation needs a building-level source fit; no deduction was included for that building.",
    ),
    (
        "ISSUE_CREATIVE_TRANSFER_OWNER_NEEDED",
        "This creative-property transfer requires separate transfer treatment before the complete estimate can be calculated.",
    ),
    (
        "RESEARCH_PROPERTY_SOURCE_INVALID",
        "Select an actual saved property of this business. Keep its disposition on that property once, and update or remove this link before deleting or moving the property.",
    ),
    ("RESEARCH_PROPERTY_CHOICE", "{0}"),
    (
        "AUTO_RECOVERY_HISTORY_INCONSISTENT",
        "Reconcile the complete annual vehicle recovery history with the original schedule and prior depreciation total.",
    ),
    ("AMORTIZATION_DATE_VALUE", "{0}"),
    ("TIMBER_START_DATE", "Section 194 amortization begins {0}."),
    (
        "TIMBER_COHORT_ONCE",
        "Enter one consolidated direct cost cohort per timber property and expenditure year across this return. Do not repeat its original elections or prior recovery.",
    ),
    (
        "TIMBER_PROPERTY_EXPENSE_LIMIT",
        "The current timber property expense election plus other elections for the same property exceeds $10,000 ($5,000 if married filing separately). Correct the actual election; excess is not automatically amortized.",
    ),
    (
        "ISSUE_TIMBER_ORIGINAL_HISTORY_NEEDED",
        "Confirm qualified U.S. commercial timber and complete original elections, unchanged basis/use and actual prior recovery before this cohort can be calculated.",
    ),
    (
        "ISSUE_TIMBER_DISPOSITION_NEEDED",
        "This timber disposition needs its gain, loss and section 1245 expense/amortization recapture calculation. The complete return estimate is unavailable.",
    ),
    (
        "LEASE_ENDS_BEFORE_BEGAN",
        "Enter valid lease dates, with the end after the beginning and no later than December 31, 2026.",
    ),
    (
        "LEASE_BEFORE_2018",
        "A vehicle lease begun before 2018 needs its earlier inclusion table and is not calculated. Enter only supported lease dates.",
    ),
    (
        "LEASE_TERM_UNDER_30_DAYS",
        "Enter a rental of fewer than 30 days on the rent expense line, rather than as a leased vehicle.",
    ),
    (
        "FARM_METHOD_CHANGE_LINK_UNAVAILABLE",
        "This accounting-method transition needs an accepted, identified Form 3115 source owned by this farm.",
    ),
    (
        "BUSINESS_RISK_DIVIDED_SOURCE_REQUIRED",
        "A divided business needs each person's actual at-risk investment and counterpart self-employment source before its limited loss can be calculated.",
    ),
    (
        "BUSINESS_RISK_SECTION179_SOURCE_REQUIRED",
        "The at-risk loss and section 179 income limit need the business's coordinated before-section-179 source.",
    ),
    ("LABEL_VEHICLE_N", "Vehicle {0,number,0}"),
    ("LABEL_ASSET_N", "Asset {0,number,0}"),
    (
        "K1_AMT_179_CARRY_BOUNDS",
        "Enter the actual AMT income-limit carryover and a sold-property part no greater than that carryover.",
    ),
    (
        "K1_179_VINTAGE_TOTAL",
        "Allocate the remaining income-limit section 179 carryover, after removing the sold-property part, to its original election years. The amounts must equal that remaining carryover.",
    ),
    ("K1_179_VINTAGE_TWICE", "Enter each original section 179 election year once."),
    (
        "K1_179_VINTAGE_BOUNDS",
        "Each original election year must be 1958 through 2025 and each remaining amount must be greater than zero.",
    ),
    (
        "SECTION179_CARE_EARNED_SOURCE_REQUIRED",
        "Dependent-care benefits with this income source need further coordinated section 179 worksheets. Review the original income, payroll and dependent-care records.",
    ),
    (
        "SECTION179_REGULAR_CARE_SOURCE_REQUIRED",
        "The section 179 AMT calculation needs the dependent-care compensation from the same return's regular before-section-179 worksheet.",
    ),
    (
        "ISSUE_SECTION179_DIRECT_AT_RISK",
        "This direct business loss needs its at-risk admission stage coordinated with the selected section 179 allocation. Retain the original source and at-risk records.",
    ),
    (
        "ISSUE_SECTION179_PTP_SOURCE_UNAVAILABLE",
        "This publicly traded partnership's selected section 179 allocation and passive-loss source records do not establish an available result. Retain its separate source worksheets.",
    ),
    (
        "ISSUE_SECTION179_OWNER_DEDUCTIONS",
        "{0} section 179 or an income-limited section 179 combined with partner health-insurance or retirement deductions still needs verified income-limit coordination.",
    ),
    (
        "EXPLAIN_SECTION179_PASSIVE_ALLOWED",
        "Section 179 deduction of this passive activity that the passive-loss limit (Form 8582) allows this year: Schedule E line 28 column (j).",
    ),
    (
        "EXPLAIN_SECTION179_PASSIVE_UNALLOWED",
        "Section 179 deduction of this passive activity held back by the passive-loss limit (Form 8582), in proportion to the activity's losses. It carries forward as its own section 179 entry of the activity's unallowed losses and is not qualified business income until allowed.",
    ),
    (
        "ISSUE_SECTION179_PUBLICLY_TRADED",
        "{0} section 179 of a publicly traded partnership follows the PTP rules for Form 4562 line 12, which this App does not calculate.",
    ),
    (
        "ISSUE_SECTION179_RISK_HISTORY",
        "{0} section 179 is tested against your remaining amount at risk (Form 6198), which needs your 2025 Form 6198 figures (Your at-risk history on the K-1).",
    ),
    (
        "ISSUE_SECTION179_RISK_CARRYFORWARD",
        "{0} section 179 above the activity's remaining amount at risk (Form 6198 line 21) depends on the Form 4562 business income limit, which itself depends on the at-risk limit; this App does not calculate that case.",
    ),
    (
        "ISSUE_A_PASSENGER_AUTOMOBILE_S_SECTION_179_ELECTION_CANNOT_EXCEED_ITS_FIR_581FCDA1",
        "A passenger automobile's section 179 election cannot exceed its first-year depreciation limit on the business-use share, and an income-limited section 179 carryforward on a passenger automobile is not calculated (Form 4562 line 26(i)).",
    ),
    (
        "ISSUE_CONFIRM_COMPLETE_SECTION_195_START_UP_COSTS_THE_ORIGINAL_DEDUCTION_D6F699EB",
        "Confirm complete section 195 start-up costs, the original deduction/capitalization election and an active business begun after October 22, 2004 and continuing through 2026. Disposition, missing prior deductions or other special treatment needs its own calculation.",
    ),
    (
        "ISSUE_GUARANTEED_PAYMENTS_SCHEDULE_K_1_1065_BOXES_4A_AND_4B_WHERE_THE_FOR_3C50E6B6",
        "Guaranteed payments (Schedule K-1 (1065) boxes 4a and 4b) where the Form 4562 line 11 business income limit binds need an established treatment as active-business compensation, which this App does not calculate.",
    ),
    (
        "ISSUE_INCOME_LIMITED_SECTION_179_WITH_ADOPTION_OR_CARE_BENEFITS_NEEDS_THE_58B6F42A",
        "Income-limited section 179 with adoption or care benefits needs the coordinated employee-compensation worksheet.",
    ),
    (
        "ISSUE_MARRIED_SEPARATE_SECTION_179_NEEDS_THE_SPOUSE_S_ELIGIBLE_PURCHASES_C6523EBC",
        "A Schedule K-1 section 179 amount on a married-separate return needs the spouse's eligible purchases and agreed dollar-limit allocation; without them the amount is lost while still reducing basis, which is not calculated.",
    ),
    (
        "ISSUE_RENTAL_FOREIGN_OR_OTHER_PROPERTY_INCOME_WITH_SECTION_179_NEEDS_ACTI_F72A67AD",
        "Rental, foreign or other property income with section 179 needs active-business income classification and coordination.",
    ),
    (
        "ISSUE_SECTION_179_COMBINED_WITH_OWNER_HEALTH_INSURANCE_OR_RETIREMENT_DEDU_0C0CC3A2",
        "Section 179 combined with owner health-insurance or retirement deductions still needs verified income-limit coordination.",
    ),
    (
        "ISSUE_SECTION179_WAGERING_CURRENT_WORK",
        "Professional wagering income needs established current management or operating work for the section 179 income limit. Historical or deemed material participation alone does not establish that work; combined spousal work on a separate return does not establish your individual work.",
    ),
    (
        "ISSUE_SECTION179_WAGERING_OWN_DEDUCTION",
        "Professional wagering income with its own section 179 election or carryforward, or a home-office deduction, needs coordinated wagering-expense treatment, which is not calculated.",
    ),
    (
        "ISSUE_SECTION179_WAGERING_PROPERTY",
        "Professional wagering income with a business sale, asset disposition, partial disposition or a fall in business use needs the checked wagering-property source, which is not calculated.",
    ),
    (
        "ISSUE_SECTION179_SEPARATE_OWN_WORK",
        "Section 179 on a separate return needs your own work in this business; not calculated.",
    ),
    (
        "ISSUE_SECTION179_DIRECT_CURRENT_WORK",
        "This business or farm needs established current management or operating work for the section 179 income limit. Historical or deemed material participation alone does not establish that work; combined spousal work on a separate return does not establish your individual work.",
    ),
    (
        "ISSUE_SECTION179_DIRECT_PAL_INCOME",
        "Current work is established, but this business or farm needs its native passive classification and allowed-loss source certified before its taxable income can enter the section 179 income limit. Operating profit alone does not establish that amount; any passive section 179 deduction needs its separate passive-loss treatment.",
    ),
    (
        "ISSUE_THE_ASSET_ELECTIONS_AND_SCHEDULE_K_1_SECTION_179_AMOUNTS_EXCEED_THE_CE43F95A",
        "The asset elections and Schedule K-1 section 179 amounts exceed the 2026 shared dollar limit after the eligible-purchase phaseout (Form 4562 line 5). Reduce your own elected costs; the excess cannot become an income-limit carryforward, and a K-1 share above the limit is lost while still reducing basis (Regulations sections 1.179-2(b)(3) and 1.179-3(a)), which this App does not calculate.",
    ),
    ("RECOVERY_HALF_YEAR", "the half-year convention"),
    ("RECOVERY_MID_QUARTER", "the mid-quarter convention"),
    ("RECOVERY_MID_MONTH", "the mid-month convention"),
    ("RECOVERY_ACTUAL_MONTHS", "actual service months"),
    (
        "RENTAL_INTEREST_ELECTION_ADS_HISTORY_UNAVAILABLE",
        "This elected rental building needs its actual ADS recovery and conversion history before its depreciation, basis or sale can be computed.",
    ),
    (
        "ASSET_MACRS_SOURCE_FACTS",
        "The recovery source records conflict with this property's actual dates, classification, use or election. Correct the source records.",
    ),
    (
        "ASSET_ADS_SOURCE_UNAVAILABLE",
        "The property's actual ADS restrictions or annual foreign-use records are unavailable. No depreciation benefit is included for the unresolved source.",
    ),
    (
        "ASSET_ADS_CLASS_LIFE_UNAVAILABLE",
        "The actual Table B classification and ADS class life are unavailable. No depreciation benefit is included for the unresolved source.",
    ),
    (
        "ASSET_QIP_SOURCE_CONTEXT_UNAVAILABLE",
        "This improvement needs its actual physical-building recovery facts and property classification. No depreciation benefit is included for the unresolved improvement.",
    ),
    (
        "ASSET_NONMACRS_AMT_HISTORY",
        "This historical accelerated property requires its actual separate straight-line AMT source; the regular allowance cannot supply it.",
    ),
    (
        "ASSET_NONMACRS_DISPOSITION_HISTORY",
        "This disposition needs the actual creator and ownership history to establish its income character. The retained recovery facts alone do not establish it.",
    ),
    (
        "ASSET_SOFTWARE_OTHER_OWNER",
        "Business-acquisition software belongs with purchased section 197 intangibles; software development belongs with research costs. Correct its source classification.",
    ),
    (
        "ASSET_FORECAST_LOOKBACK_UNAVAILABLE",
        "This income-forecast property requires its original Form 8866 tax and annual source comparisons. Current forecast income cannot establish the mandatory lookback tax.",
    ),
    (
        "FORECAST_SMALL_COST_EXCEPTION",
        "The income-forecast lookback cost exception requires total capitalized cost of no more than $100,000, before depreciation, use allocation or credit reductions.",
    ),
    (
        "FORECAST_NOT_WITHIN_TEN_PERCENT",
        "The actual and originally forecast income amounts differ by more than10%. Correct the source amounts or retain the required lookback worksheet.",
    ),
    (
        "METHOD_CHANGE_NOTHING_IN_CURRENT_YEAR",
        "Nothing of this adjustment falls in 2026: a decrease or a one-year increase is taken in its year of change, a two-year increase in that year and the next, and a four-year increase in that year and the next three.",
    ),
    (
        "METHOD_CHANGE_DE_MINIMIS_LIMIT",
        "The one-year de minimis election applies only to an increase under $50,000.",
    ),
    (
        "METHOD_CHANGE_OUT_OF_LIMITS",
        "Enter the accounting-method change within the supported limits.",
    ),
    ("MESSAGE_ENTER_AT_MOST_10_METHOD_CHANGES", "Enter at most 10 accounting-method changes."),
    ("METHOD_CHANGE_DCN", "DCN {0}"),
    ("METHOD_CHANGE_NO_DCN", "no DCN"),
    (
        "NOTE_METHOD_CHANGE_FORM_3115",
        "{0}: the section 481(a) adjustment of the {1,number,0} accounting-method change ({2}) assumes a Form 3115 filed under its consent terms. The App does not prepare Form 3115 and does not apply the section 481(b) limitation on tax, which can only lower it.",
    ),
    (
        "RENTAL_1993_RECOVERY_SOURCE_REQUIRED",
        "For commercial property first available from May 13 through December 31, 1993, answer the actual historical contract or construction source. Do not assume a recovery period.",
    ),
    (
        "RENTAL_1993_TRANSITION_NOT_APPLICABLE",
        "The 1993 transition source applies only to nonresidential property first available from May 13 through December 31, 1993. Clear it for this property type or service date.",
    ),
    (
        "RENTAL_1993_TRANSITION_SOURCE_DATE",
        "Enter the actual binding-contract or construction-commencement date before May 13, 1993.",
    ),
    (
        "RENTAL_1993_TRANSITION_SOURCE_REFERENCE",
        "Enter a reference identifying the actual contract, construction or qualifying transfer record.",
    ),
    ("DISPLAY_LEASE_TERM", "Lease and payments"),
    ("DISPLAY_LEASE_USE", "Use and operating costs"),
    ("DISPLAY_LEASE_RECORDS", "Vehicle records"),
    (
        "INTANGIBLE_PREDECESSOR_INVALID",
        "Choose an actual earlier intangible in this same activity, or leave the predecessor unlinked. A missing, duplicate, other-source or later record cannot identify the predecessor.",
    ),
    (
        "INTANGIBLE_MORTGAGE_SOURCE",
        "Mortgage servicing rights need the actual servicing interest, residential-business-acquisition facts and any stripped-coupon treatment before section 197 is determined. This broad category alone does not establish eligibility or exclusion. Preserve the finite-life property source when its statutory exception applies, entering the cost once.",
    ),
    (
        "INTANGIBLE_FIXED_RIGHT_SOURCE",
        "This fixed-term right needs the actual underlying property or service right before its section 197 treatment is determined. A whole-business acquisition does not override an excluded right to receive tangible property or services. Preserve its actual finite-life, lease or property source without duplicating cost.",
    ),
    (
        "INTANGIBLE_SOFTWARE_SOURCE",
        "Classify this software in the existing non-MACRS property source: purchased software, hardware-bundled software, or research/development. Business-acquisition and mass-market software need their actual statutory facts. Move its cost once; do not deduct it in both sources.",
    ),
    (
        "INTANGIBLE_CREATIVE_SOURCE",
        "Separately acquired film, book and recording rights retain their existing finite-life or income-forecast source, including its original useful-life and election facts. Correct this record through normal editing and enter its cost once in that source.",
    ),
    (
        "INTANGIBLE_OTHER_OWNER",
        "Financial interests, land rights and existing tangible-property leases need their actual investment, property or lease source; section 197 does not supply their deduction. Correct this source and enter the cost only once.",
    ),
    (
        "INTANGIBLE_FINITE_SOURCE",
        "This separate government, patent/copyright/know-how, mortgage-servicing or fixed-term right needs the specific statutory exception and useful-life facts. The broad category alone does not resolve section 197. Preserve the existing finite-life non-MACRS alternative when the actual right is excluded, entering its cost once.",
    ),
    (
        "INTANGIBLE_CLASSIFICATION",
        "This asset or right needs its statutory classification before a section 197 deduction can be determined.",
    ),
    (
        "INTANGIBLE_NONCOMPETE_TRANSACTION",
        "A covenant not to compete needs its actual connection with a business or business-interest acquisition. The recorded separate-right facts do not establish that connection.",
    ),
    (
        "INTANGIBLE_SELECTED_ASSETS",
        "The acquired and seller-retained operating assets need the substantial-business-portion analysis; selected assets are not automatically admitted or excluded.",
    ),
    (
        "INTANGIBLE_OWNERSHIP_INTEREST",
        "A business ownership interest is not an amortizable acquisition of its underlying intangible assets. Any deemed-asset-purchase treatment needs its actual election and source records.",
    ),
    (
        "INTANGIBLE_SELF_CREATED",
        "This separately self-created intangible is outside ordinary section 197 amortization. Use its actual research or finite-life source where applicable, preserving cost once; no useful life or election is assumed.",
    ),
    (
        "INTANGIBLE_HISTORIC_SOURCE",
        "Historical or unproved origin needs transition-period holders/users, relationships, transaction-series and applicable seller/depreciability exceptions. A new acquisition or renewal date alone does not resolve anti-churning; a pre-1993 election is not assumed.",
    ),
    (
        "INTANGIBLE_TRANSFER_SOURCE",
        "This gift, inheritance, entity transaction or exchange needs its actual carryover/stepped-basis and transfer records before section 197 recovery can be determined.",
    ),
    (
        "INTANGIBLE_PAYMENT_SOURCE",
        "Contingent principal/interest, recurring-use payments and inclusion timing need the actual agreement and accounting sources before this deduction is determined.",
    ),
    (
        "INTANGIBLE_LEASE_SOURCE",
        "The recorded lease needs its actual counterparty and full statutory term, including options, renewals and related arrangements, before any section 197 tax-exempt-use floor is determined. Leased does not mean ineligible.",
    ),
    (
        "INTANGIBLE_LEASE_UNAVAILABLE",
        "The lease records are unavailable; this intangible's section 197 deduction remains undetermined.",
    ),
    (
        "INTANGIBLE_SOURCE_DATES",
        "Acquisition, business commencement and all entered cost-inclusion dates must reach the return year before this year's deduction is determined.",
    ),
    (
        "INTANGIBLE_EVENTS_UNAVAILABLE",
        "Basis, use and transfer records are unavailable; absence of events cannot be assumed.",
    ),
    (
        "INTANGIBLE_RECORDED_EVENTS",
        "The recorded decrease, changed use or transfer needs its dated amount, use and transaction source before recovery is determined. A documented acquisition-date price correction belongs once in original cost; later reductions cannot be treated as that correction.",
    ),
    (
        "INTANGIBLE_DISPOSITION_GROUP",
        "This actual disposition needs the original clock's disposal-month cutoff, actual prior recovery, retained-intangible loss reallocation and grouped recapture before per-asset gain or loss can be recognized.",
    ),
    (
        "INTANGIBLE_USE_UNAVAILABLE",
        "Disposition or continued business-use records are unavailable; continued ownership and use cannot be assumed.",
    ),
    (
        "INTANGIBLE_PRIOR_UNAVAILABLE",
        "Actual pre-return-year regular and AMT deductions are needed from earlier returns and basis records, including all additions.",
    ),
    (
        "INTANGIBLE_PRIOR_CORRECTION",
        "Actual earlier regular or AMT deductions differ from the original statutory schedule. Their correction and adjusted-basis treatment need the actual recovery-history source; no catch-up deduction is assumed.",
    ),
];
