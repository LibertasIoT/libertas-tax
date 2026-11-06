pub(crate) const FARM_STRINGS: &[(&str, &str)] = &[
    (
        "HOME_FARM_PREPAID_SOURCE_UNAVAILABLE",
        "The farm prepaid ceiling depends on an unavailable home-use worksheet. Resolve its actual source records before calculating the coupled deductions.",
    ),
    (
        "FARM_FERTILIZER_SOURCE",
        "Supply actual capital fertilizer cost, payment/incurrence and application dates and retained original tax treatment. Prior recovered basis cannot exceed cost or precede a current-year cost.",
    ),
    (
        "NOTE_FARM_FERTILIZER_RECOVERY_OMITTED",
        "No capital fertilizer recovery deduction is included without its actual original recovery method and history. Benefit months alone do not establish a monthly convention. Its resulting remaining basis also needs the actual affected property before a sale can be calculated.",
    ),
    (
        "NOTE_FARM_FERTILIZER_180_NOT_ELIGIBLE",
        "Section 180 does not deduct nonfarming costs or initial preparation of previously unfarmed land. Known farm receipts are retained. Capital property basis needs its actual property and recovery owner.",
    ),
    (
        "ISSUE_FARM_PROPERTY_DISPOSITION_HISTORY",
        "Resolve the actual disposition's basis and ownership history; required gain is not omitted with an unclaimed deduction.",
    ),
    (
        "FARM_OTHER_SOURCE_REQUIRED",
        "Describe the actual additional farm source only when Other is selected.",
    ),
    (
        "ISSUE_FARM_FOREIGN_SOURCE",
        "Foreign farm income needs its actual currency, source and cross-border tax adjustments.",
    ),
    (
        "ISSUE_FARM_ENTITY_SOURCE",
        "Use the entity/K-1 and required accounting owner for this entity or farming syndicate.",
    ),
    (
        "ISSUE_FARM_METHOD_TRANSITION_SOURCE",
        "Resolve this farm's actual accounting transition, consent and section 481 timing source.",
    ),
    (
        "ISSUE_FARM126_ECONOMIC_VALUE_SOURCE",
        "Supply the actual improvement economic fair market value; missing optional exclusion appraisal alone belongs on its improvement record.",
    ),
    (
        "ISSUE_FARM_INTEREST_ALLOCATION_SOURCE",
        "Resolve the actual indirect-interest expense, capitalization and property allocation.",
    ),
    (
        "ISSUE_FARM_HEDGE_ALLOCATION_SOURCE",
        "Resolve the complex hedge's matched income, cost character and timing.",
    ),
    (
        "ISSUE_FARM_ADDITIONAL_SOURCE",
        "Resolve the described additional farm income, timing or property source; known receipts remain included.",
    ),
    ("FARM_METHOD_CHANGE_CHOICE", "{0} — {1} accounting change"),
    (
        "FARM126_COST_REQUIRED",
        "Enter actual nonnegative improvement funding and a positive total cost.",
    ),
    (
        "FARM126_OWNER_ABOVE_COST",
        "Actual owner and government amounts cannot exceed the total improvement cost.",
    ),
    (
        "FARM126_COMPONENTS_ABOVE_COST",
        "The disjoint noneligible, uncertified, rent or service and current-deduction components cannot exceed total cost.",
    ),
    (
        "FARM126_FUNDING_WITHOUT_EXCLUSION",
        "Excluded funding applied must be zero when the program is not approved, conservation certification is absent, or you elected not to exclude any payment.",
    ),
    (
        "FARM126_EXCLUDED_FUNDING_ABOVE_SOURCE",
        "Excluded funding applied cannot exceed this improvement's cost or the government payment received.",
    ),
    (
        "FARM126_SOURCE_REQUIRED",
        "Enter the actual improvement, program, installation year, current economic-benefit receipt date and affected property source.",
    ),
    (
        "FARM126_PRIOR_INCOME_YEARS",
        "Enter gross receipts for the affected acreage in each of the three years before installation began, once each.",
    ),
    (
        "FARM126_VALUATION_SOURCE",
        "Use supported nonnegative income-right present values and the actual appraisal assumptions.",
    ),
    (
        "FARM126_PARTIAL_ELECTION_AMOUNT",
        "The partial election amount cannot exceed the supported otherwise excludable portion.",
    ),
    (
        "FARM_LAND_PART_ALLOCATION",
        "The recorded amount must be nonnegative. Amounts allocated to all sold tracts together cannot exceed the selected parcel's corresponding opening amount.",
    ),
    (
        "ISSUE_FARM_LAND_PART_ALLOCATION_NEEDED",
        "Enter the sold tract's actual regular and AMT opening basis and prior conservation deductions. Partial sales with parcel-wide cost-sharing, current conservation or cooperative adjustments cannot yet be calculated because those source records do not identify the sold tract. No fraction is assumed.",
    ),
    (
        "ISSUE_FARM_LAND_TRANSACTION_OWNER_NEEDED",
        "This sale needs its actual transaction owner for partial, related-party, exchange, special-basis or unresolved transfer facts. Required gain cannot be omitted as an unclaimed benefit.",
    ),
    (
        "FARM126_PROPERTY_SOURCE",
        "Select an accepted parcel belonging to this farm and acquired no later than the improvement was received. Save and reopen a new parcel before linking it.",
    ),
    (
        "ISSUE_FARM126_MIXED_SOURCE_OWNER",
        "Mixed payment character or incomplete improvement funding needs the actual excluded cost-share funding applied to this improvement. Supply that source amount; the income-right present value does not establish the funding allocation.",
    ),
    (
        "NOTE_FARM126_VALUATION_OMITTED",
        "The section 126 income-right appraisal is unavailable, so the estimate applies no exclusion and does not establish a recapture balance. The ordinary fully funded worksheet cannot establish basis without it; mixed funding uses the separately entered actual excluded funding allocation. Supply the actual appraisal, or choose an actual timely no-exclusion election if that is what you made.",
    ),
    ("DISPLAY_FARM126_PROPERTY", "Improvement and property"),
    ("DISPLAY_FARM126_FINANCING", "Actual financing"),
    ("DISPLAY_FARM126_EXCLUSION", "Exclusion or election"),
    ("DISPLAY_FARM_WEATHER_CLASS", "Livestock class"),
    ("DISPLAY_FARM_WEATHER_SALES", "Class sales and income"),
    ("DISPLAY_FARM_WEATHER_STATEMENT", "Weather election statement"),
    ("DISPLAY_FARM_SPECIAL_INCOME", "Income and elections"),
    (
        "FARM_CONSERVATION_METHOD_CONFLICT",
        "The same taxpayer cannot have both a never-elected section 175 history and a known prior deduction or established deduction method. Check the original returns or actual IRS consent covering a different farm or project.",
    ),
    (
        "FARM_SUPPLY_PURCHASE_TERMS",
        "A claimed binding purchase needs its actual quantity, price and delivery terms and a business purpose beyond reducing tax. If terms are unestablished, choose that answer instead of claiming a current unused-supply deduction.",
    ),
    (
        "FARM_SUPPLY_REFUND_ABOVE_PAYMENT",
        "The refund of this current unused payment cannot exceed its original paid amount. A recovery of a prior deducted payment needs its separate income source.",
    ),
    (
        "FARM_CONSERVATION_METHOD_HISTORY",
        "Carried section 175 deductions and prior district payments cannot predate the claimed original deduction election. Check retained source years and original returns; a later change from capitalization needs its actual IRS consent.",
    ),
    (
        "FARM_LAND_DUPLICATE_PARCEL",
        "Enter a whole-parcel disposition once. A partial sale needs its actual allocated parcel and basis records; do not reuse the whole parcel for more than one sale.",
    ),
    ("DISPLAY_FARM_CONSERVATION_PROPERTY", "Parcel and qualifying work"),
    ("DISPLAY_FARM_CONSERVATION_ASSESSMENT", "District assessment and payments"),
    ("DISPLAY_FARM_SPECIAL_COSTS", "Production and fertilizer costs"),
    ("DISPLAY_FARM_SPECIAL_HISTORY", "Source records and tax situations"),
    ("DISPLAY_FARM_PRODUCTION_PROPERTY", "Produced property"),
    ("DISPLAY_FARM_PRODUCTION_COSTS", "Method and costs"),
    ("DISPLAY_FARM_PRODUCTION_BASIS", "Basis and destination"),
    (
        "FARM_AVERAGING_SOURCES_INVALID",
        "Select each actual accepted farming or fishing source only once.",
    ),
    (
        "FARM_WEATHER_INCOME_SPLIT",
        "The raised-for-sale portion cannot exceed this record's class income.",
    ),
    ("FARM_SPECIALS_LIST_LIMIT", "Use the supported number of distinct farm source records."),
    (
        "FARM_METHOD_ADOPTED_YEAR",
        "Enter the actual year this farm adopted its original accounting method.",
    ),
    ("FARM_LIMITATIONS_INVALID", "Choose each actual named farm situation once."),
    (
        "FARM_WEATHER_DUPLICATE_CLASS",
        "Combine this farm's source records within each generic livestock class; do not subdivide by age, breed or sex.",
    ),
    ("FARM_OTHER_SOURCE", "Check the actual ordinary farm other-income source and signed amount."),
    (
        "FARM_LAND_BASIS_SOURCE",
        "Check the parcel identity, acquisition date, regular/AMT basis and retained conservation/exclusion history.",
    ),
    (
        "FARM_WEATHER_ABOVE_PRODUCT_INCOME",
        "Class income must already be included once in the matching raised or purchased product receipts; exclude business-use animal sales.",
    ),
    (
        "ISSUE_FARM_WEATHER_CURRENT_CHARACTER",
        "Current class income from business-use livestock needs its retained Form 4797 character, basis and asset source. It cannot be omitted as an unclaimed postponement or assumed to be ordinary product income.",
    ),
    (
        "NOTE_FARM_WEATHER_NOT_POSTPONED",
        "No weather postponement was applied. The claim needs cash accounting, principal farming, federal assistance designation and supported product income character. Known income remains current; business-use or combined activity source allocation needs its own owner.",
    ),
    (
        "ISSUE_FARM_WEATHER_PRIOR_CHARACTER",
        "Prior postponed income from business-use livestock needs its retained Form 4797 character and asset source. It is not ordinary product income by assumption.",
    ),
    (
        "ISSUE_FARM_HEDGE_TIMING_CHARACTER",
        "An unidentified, inventory-matched or multiyear hedge needs its actual income character and timing owner. A realized amount alone does not establish ordinary current income or an investor60/40 result.",
    ),
    (
        "ISSUE_FARM_PRODUCTION_BASIS_OWNER",
        "Preproductive costs need the linked inventory or depreciable property's regular and AMT basis, release and recovery facts; a paid cost cannot be both expensed and capitalized.",
    ),
    (
        "FARM_WEATHER_COUNTS",
        "Check actual total, normal and weather-forced animal counts and the class income.",
    ),
    (
        "FARM_WEATHER_SOURCE",
        "Supply the weather relationship and usual-practice evidence from actual records.",
    ),
    (
        "FARM_WEATHER_DESIGNATION_DATE",
        "Check the actual federal assistance designation date, or leave the unknown date absent.",
    ),
    (
        "FARM_WEATHER_YEARS",
        "Use distinct actual years within the preceding three years; a new farm can use regional usual-practice evidence.",
    ),
    (
        "FARM_HEDGE_SOURCE",
        "Check the hedge identification, entry and settlement dates, ordinary business risk and source amount.",
    ),
    (
        "FARM_AVERAGING_SELECTION_INVALID",
        "Selected capital must not exceed the total election; selected unrecaptured gain must not exceed selected capital. Use distinct valid accepted source identities.",
    ),
    (
        "WARN_FARM_AVERAGING_PRODUCTION_SOURCE_LEFT_OUT",
        "Farm income averaging is left out because service or purchased-resale receipts need their actual production character and attributable net income. Contract harvesting and merely reselling others' products are not eligible by assumption. Ordinary farm income remains calculated.",
    ),
    (
        "WARN_FARM_AVERAGING_HISTORY_LEFT_OUT",
        "Farm income averaging is left out because the selected base-year history needs its separate spouse, NOL, MSA, minor-filer, short-year or foreign worksheet. Known ordinary tax remains calculated.",
    ),
    (
        "WARN_FARM_AVERAGING_CAPITAL_SOURCE_LEFT_OUT",
        "The selected farm capital-gain election needs the disposition owner's final attributable net gain and unrecaptured gain. Farm income averaging is left out; ordinary tax retains the actual income and gain.",
    ),
    (
        "WARN_FARM_AVERAGING_ADDITIONAL_SOURCE_LEFT_OUT",
        "Additional linked farming or fishing sources need their final taxable-income and deduction attribution. Farm income averaging is left out; known ordinary income remains calculated.",
    ),
    (
        "WARN_FARM_AVERAGING_ALLOCATION_LEFT_OUT",
        "Farm income averaging is left out because the current deduction, gain, QBI or spouse-history allocation needs its separate source worksheet. Known ordinary tax remains calculated.",
    ),
    (
        "ISSUE_FARM_CURRENT_POULTRY_TIMING_SOURCE",
        "Current production poultry needs its actual cash-payment method, business-use start and lesser-of-12-months/useful-life allocation. Partial-month or earlier starts and depreciation/method changes need their retained source worksheet; no current expense or prepaid percentage is guessed.",
    ),
    ("DISPLAY_FARM_CURRENT_ELECTION", "Current election"),
    ("DISPLAY_FARM_BASE_THIRD_PRIOR_YEAR", "2023 return"),
    ("DISPLAY_FARM_BASE_SECOND_PRIOR_YEAR", "2024 return"),
    ("DISPLAY_FARM_BASE_PRIOR_YEAR", "2025 return"),
    ("DISPLAY_FARM_BASE_RETURN", "Retained return"),
    ("DISPLAY_FARM_BASE_PREFERENCES", "Capital, foreign and investment amounts"),
    (
        "CROP_PRACTICE_SOURCE",
        "Enter one retained normal-sales-practice record for each identified damaged crop, without duplicate crop records.",
    ),
    (
        "CROP_PRACTICE_ABOVE_TOTAL",
        "The normal crop income belonging in following years cannot exceed its actual normal total income.",
    ),
    (
        "WARN_CROP_PRACTICE_DEFERRAL_LEFT_OUT",
        "Crop-insurance deferral was left out; current proceeds remain income. This calculation requires cash accounting, current-year physical loss, and records showing more than half of every affected crop's normal income in a following year. Mixed sales practices need their separate election eligibility treatment.",
    ),
    (
        "CCC_PRODUCTION_BASIS_SOURCE",
        "Enter the actual nonnegative unrecovered production cost for the pledged commodity.",
    ),
    (
        "CCC_ELECTION_HISTORY_SOURCE",
        "Enter the original section 77 election or change year and retained source statement.",
    ),
    (
        "ISSUE_CCC_COMMODITY_BASIS_SOURCE",
        "The CCC commodity needs its actual ownership and unrecovered production-cost basis. The zero-basis raised-crop calculation cannot replace those facts or discard known loan income.",
    ),
    (
        "ISSUE_CCC_ELECTION_TRANSITION_SOURCE",
        "Original section 77 history must agree with the current method and earlier included loans. A documented method change needs its actual transition and commodity-basis owner; changing the method answer alone cannot compute it.",
    ),
    (
        "FARM_PRODUCTION_DATES",
        "Check the identified production record, actual production/service dates and retained source basis.",
    ),
    (
        "FARM_PRODUCTION_METHOD_SOURCE",
        "Enter the actual original uniform-capitalization election year and supporting statement.",
    ),
    (
        "FARM_PRODUCTION_EXPENSE_OWNER",
        "The de minimis election excludes inventory and section 263A costs. Enter independently eligible purchases in ordinary business expenses; use actual ordinary, meal or nondeductible treatment for production costs.",
    ),
    (
        "FARM_PRODUCTION_ALLOCATION",
        "Production cost allocations must be nonnegative, disjoint, and no greater than their identified paid or incurred cost.",
    ),
    (
        "FARM_PRODUCTION_RELEASE",
        "Released inventory cost cannot exceed its actual available unrecovered regular or AMT basis.",
    ),
    (
        "NOTE_FARM263A_EXEMPTION_NOT_MET",
        "The entered receipt history or tax-shelter facts do not meet the small-business uniform-capitalization exemption. Actual required capital allocations remain in basis.",
    ),
    (
        "FARM263A_RECEIPT_YEARS",
        "Supply the actual statutory receipt-test years, months and aggregated gross receipts, including the required predecessor history.",
    ),
    ("FARM_LIMITATION_1", "Foreign farm source"),
    ("FARM_LIMITATION_2", "Entity or farming syndicate"),
    ("FARM_LIMITATION_3", "Unentered accounting-method transition"),
    ("FARM_LIMITATION_4", "Unfinished improvement economic fair market value"),
    ("FARM_LIMITATION_5", "Revised district assessment"),
    ("FARM_LIMITATION_6", "Indirect-interest allocation"),
    ("FARM_LIMITATION_7", "Basis or ownership transfer"),
    ("FARM_LIMITATION_8", "Complex hedge timing"),
    ("FARM_LIMITATION_9", "Missing averaging history"),
    ("FARM_LIMITATION_10", "Other required income or timing source"),
    (
        "FARM_HEADER_CODE_INVALID",
        "Choose the principal agricultural activity code from the current Schedule F list.",
    ),
    (
        "FARM_HEADER_EIN_INVALID",
        "Enter an actual nine-digit employer identification number, or leave it absent.",
    ),
    (
        "ISSUE_FARM_HEADER_REQUIRED",
        "The Schedule F principal agricultural code and information-return answers are needed.",
    ),
    ("NOTE_FARM_HEADER", "Schedule F line B: {0}. Line D EIN: {1}. Lines F/G: {2}."),
    ("FARM_INFORMATION_NOT_REQUIRED", "No information returns required"),
    ("FARM_INFORMATION_FILED", "Information returns filed or will be filed"),
    ("FARM_INFORMATION_NOT_FILED", "Required information returns not all filed"),
    (
        "NOTE_FARM_INFORMATION_INCOMPLETE",
        "Check Schedule F lines F/G: the information-return requirement or filing remains unresolved. This does not remove known farm income.",
    ),
    ("FARM_EIN_ABSENT", "Not supplied"),
    ("FARM_CODE_111100", "Oilseed and grain farming"),
    ("FARM_CODE_111210", "Vegetable and melon farming"),
    ("FARM_CODE_111300", "Fruit and tree nut farming"),
    ("FARM_CODE_111400", "Greenhouse, nursery, and floriculture production"),
    ("FARM_CODE_111900", "Other crop farming"),
    ("FARM_CODE_112111", "Beef cattle ranching and farming"),
    ("FARM_CODE_112112", "Cattle feedlots"),
    ("FARM_CODE_112120", "Dairy cattle and milk production"),
    ("FARM_CODE_112210", "Hog and pig farming"),
    ("FARM_CODE_112300", "Poultry and egg production"),
    ("FARM_CODE_112400", "Sheep and goat farming"),
    ("FARM_CODE_112510", "Aquaculture"),
    ("FARM_CODE_112900", "Other animal production"),
    ("FARM_CODE_113000", "Forestry and logging (including forest nurseries and timber tracts)"),
    ("FARM_CODE_113110", "Timber tract operations"),
    ("FARM_CODE_113210", "Forest nurseries and gathering of forest products"),
    ("FARM_CODE_113310", "Logging"),
    ("DISPLAY_COOP_QUALIFICATION", "Issuer qualification"),
    ("DISPLAY_COOP_ALLOCATION", "Expense and wage allocation"),
    (
        "COOP_ALLOCATION_ID_INVALID",
        "A cooperative share identity is missing or repeated. Reopen the income editor.",
    ),
    (
        "COOP_RECIPIENT_LINK_INVALID",
        "Choose the actual accepted business, farm or farm-rental recipient belonging to this owner.",
    ),
    (
        "COOP_PROPERTY_LINK_UNAVAILABLE",
        "This property owner has no accepted property identity in this App snapshot. Preserve the source description and leave the unavailable link empty.",
    ),
    (
        "WARN_COOP_QUALIFICATION_LEFT_OUT",
        "The qualified parts of the cooperative distribution need the actual issuer/source allocation after personal purchases, property refunds and acquired-notice basis. The unestablished QBI benefit is left out; known taxable income remains.",
    ),
    (
        "WARN_COOP_DPAD_NOTICE_NEEDED",
        "The cooperative DPAD is left out because its actual allocation notice has not been established. Known cooperative income remains.",
    ),
    (
        "WARN_COOP_PATRON_ALLOCATION_NEEDED",
        "The affected cooperative QBI benefit is left out because the actual qualified-payment wage/deduction allocation is missing or does not reconcile to the accepted source pools. Known taxable income and losses remain.",
    ),
    (
        "WARN_COOP_QBI_RECIPIENT_NEEDED",
        "The cooperative QBI/DPAD benefit needs its supported business recipient or actual pass-through QBI statement. Known source income remains.",
    ),
    (
        "WARN_COOP_CREDIT_SOURCE_NEEDED",
        "The cooperative credit benefit is left out because its actual issuer breakdown, allocation notice or accepted same-owner activity is missing. Known income and withholding remain.",
    ),
    (
        "WARN_COOP_CREDIT_PASSIVE",
        "Participation in the cooperative credit activity is not established. The existing passive-credit worksheet limits this credit; participation is not assumed.",
    ),
    (
        "WARN_COOP_OTHER_BENEFIT_LEFT_OUT",
        "The cooperative benefit {0}, from {1}, has no supported current source owner. Only that benefit is left out; any established required income is retained.",
    ),
    ("COOP_EMPOWERMENT_ZONE", "Form 8844 empowerment-zone credit"),
    ("COOP_LOW_SULFUR_DIESEL", "Form 8896 low-sulfur diesel credit"),
    ("COOP_DIFFERENTIAL_WAGE", "Form 8932 differential-wage credit"),
    ("COOP_SMALL_AGRI_BIODIESEL", "Form 8864 small-agri-biodiesel credit"),
    ("COOP_SMALL_REFINER", "Section 198 small-refiner deduction"),
    (
        "ISSUE_COOP_CREDIT_INCOME_NEEDED",
        "The actual source does not establish the required credit-related income or its accepted recipient. No required income is guessed.",
    ),
    (
        "COOP_SSTB_EXPENSE_INVALID",
        "The cooperative SSTB expense allocation exceeds the actual deductible category already entered on its business, or needs an unavailable recipient expense owner.",
    ),
    (
        "COOP_SSTB_SOURCE_REPEATED",
        "Record each actual related expense source part once for an activity; identify disjoint invoice parts explicitly.",
    ),
    (
        "COOP_PATRON_WAGES_NOT_ZERO",
        "This activity has eligible W-2 wages. Use its actual qualified-payment allocation instead of the zero-wage answer.",
    ),
    (
        "COOP_PATRON_SOURCE_DUPLICATE",
        "Keep one governing qualified-payment allocation record for each activity across all cooperative statements.",
    ),
    (
        "COOP_ALLOCATION_INVALID",
        "Correct the cooperative allocation amounts, sources and disjoint parts.",
    ),
    (
        "COOP_PROPERTY_QBI_ABOVE_AMOUNT",
        "The property's qualified and SSTB portions cannot exceed its refund amount.",
    ),
    (
        "COOP_PROPERTY_PAYMENTS_ABOVE_AMOUNT",
        "The property's qualified payments cannot exceed its refund amount.",
    ),
    (
        "COOP_PERSONAL_ABOVE_BOX1",
        "The personal portion cannot exceed this source's box 1 distribution.",
    ),
    (
        "COOP_PERSONAL_ABOVE_BOX2",
        "The personal portion cannot exceed this source's box 2 distribution.",
    ),
    (
        "COOP_PERSONAL_ABOVE_BOX3",
        "The personal portion cannot exceed this source's box 3 distribution.",
    ),
    (
        "COOP_PERSONAL_ABOVE_BOX5",
        "The personal portion cannot exceed this source's box 5 distribution.",
    ),
    (
        "COOP_ALLOCATIONS_DO_NOT_RECONCILE",
        "The recipient allocations must reconcile exactly to the printed distributions and boxes 6 through 9.",
    ),
    (
        "COOP_CREDIT_SOURCE_INVALID",
        "Correct the issuer credit components, notice and recipient source facts.",
    ),
    (
        "ISSUE_COOP_NOTICE_HISTORY_NEEDED",
        "Record notice history covering the entire box 5 redeemed amount, including actual ownership, basis and stated value. Above-stated-value proceeds, losses or basis shared with an excluded part need their source allocation; no basis or gain character is guessed.",
    ),
    (
        "ISSUE_COOP_DISTRIBUTION_CHARACTER_NEEDED",
        "Purchase exclusions require the original patronage-dividend or patronage-notice source. Nonpatronage distributions and per-unit retains are not assumed to be personal-purchase refunds.",
    ),
    (
        "ISSUE_COOP_SOURCE_INVALID",
        "Correct the cooperative source amounts and component records before estimating.",
    ),
    (
        "WARN_FARM_RENTAL_PASSIVE_ASSUMED",
        "{0}: assumed you did not materially participate in this farm rental under the passive-activity rules, so its loss stays limited even though you are a real estate professional.",
    ),
    (
        "WARN_FARM_RENTAL_PROPERTY_PAL_SOURCE",
        "This property's gain is reported, but is not used to release the farm rental's passive loss. Review its property-use, tenant and development records in Farming; this estimate supports only current losses and positive gains with a nonpositive combined net and no prior losses or special activity treatment.",
    ),
    (
        "ISSUE_FARM_AMT_PREPAID_SOURCE",
        "Distinct AMT farm expenses or retained supply costs need their final prepaid allowance and any dependent home-office limit. Known receipts remain; an unresolved dependent source does not establish completed tax.",
    ),
    (
        "FARM_CONSERVATION_PROPERTY_SOURCE",
        "Select an accepted parcel of this farm for every conservation cost, assessment and carry. Save new parcel facts first, then reopen this editor to select them.",
    ),
    (
        "FARM_CONSERVATION_PAYMENT_DATE",
        "Only a current-year assessment payment can have a current-year date. Keep an earlier payment's actual tax year without inventing a day.",
    ),
    (
        "FARM_CONSERVATION_CARRY_YEAR",
        "Enter a prior source year for the unused conservation carry; current costs belong in current records.",
    ),
    (
        "FARM_CONSERVATION_CARRY_DUPLICATE",
        "Combine each parcel and source year's conservation carry in one record, retaining separate regular-tax and AMT amounts.",
    ),
    (
        "FARM_CONSERVATION_BEFORE_ACQUIRED",
        "Conservation costs and retained deductions must belong to this parcel's ownership period. Resolve predecessor or inherited history at its actual basis source.",
    ),
    (
        "FARM_CONSERVATION_PLAN_REQUIRED",
        "Identify the actual approved conservation or site-specific recovery plan, or select no approved plan.",
    ),
    (
        "WARN_FARM_CONSERVATION_FACTS_OMITTED",
        "The section 175 deduction is omitted for costs whose qualifying plan, work, land use or election is not established. Known farm income and other qualifying conservation costs remain included. Keep the omitted costs' basis and expense records; your tax may be lower.",
    ),
    (
        "ISSUE_FARM_LAND_RESOLVE_BUSINESS_PROPERTY_SECTION_1231_CHARACTER_BEFORE_7F94E201",
        "Resolve business-property section 1231 character before business QBI.",
    ),
    (
        "ISSUE_FARM_LAND_INSTALLMENT_SHORT_HOLDING_PERIOD",
        "An installment sale of farm land held one year or less is not calculated (Form 6252 line 26 to Form 4797 line 11).",
    ),
    (
        "ISSUE_FARM_LAND_INSTALLMENT_WITH_DEFERRAL",
        "An installment sale of farm land with the Form 1062 tax-payment election is not calculated (Form 6252 and Form 1062 line 15).",
    ),
    ("DISPLAY_4835_LINE9_CHEMICALS", "Line 9 chemicals"),
    ("DISPLAY_4835_LINE11_CUSTOM_HIRE", "Line 11 custom hire (machine work)"),
    (
        "DISPLAY_4835_LINE13_EMPLOYEE_BENEFITS",
        "Line 13 employee benefit programs other than on line 21",
    ),
    ("DISPLAY_4835_LINE14_FEED", "Line 14 feed"),
    ("DISPLAY_4835_LINE15_FERTILIZERS", "Line 15 fertilizers and lime"),
    ("DISPLAY_4835_LINE16_FREIGHT", "Line 16 freight and trucking"),
    ("DISPLAY_4835_LINE17_FUEL", "Line 17 gasoline, fuel and oil"),
    ("DISPLAY_4835_LINE18_INSURANCE", "Line 18 insurance, other than health"),
    ("DISPLAY_4835_LINE19A_MORTGAGE_INTEREST", "Line 19a mortgage interest paid to banks"),
    ("DISPLAY_4835_LINE19C_OTHER_INTEREST", "Line 19c other interest"),
    ("DISPLAY_4835_LINE20_LABOR", "Line 20 labor hired, less employment credits"),
    ("DISPLAY_4835_LINE21_PENSION", "Line 21 pension and profit-sharing plans"),
    (
        "DISPLAY_4835_LINE22A_EQUIPMENT_RENT",
        "Line 22a rent or lease of vehicles, machinery and equipment",
    ),
    (
        "DISPLAY_4835_LINE22B_OTHER_RENT",
        "Line 22b rent or lease of land, animals and other property",
    ),
    ("DISPLAY_4835_LINE23_REPAIRS", "Line 23 repairs and maintenance"),
    ("DISPLAY_4835_LINE24_SEEDS", "Line 24 seeds and plants"),
    ("DISPLAY_4835_LINE25_STORAGE", "Line 25 storage and warehousing"),
    ("DISPLAY_4835_LINE26_SUPPLIES", "Line 26 supplies"),
    ("DISPLAY_4835_LINE27_TAXES", "Line 27 taxes"),
    ("DISPLAY_4835_LINE28_UTILITIES", "Line 28 utilities"),
    ("DISPLAY_4835_LINE29_VETERINARY", "Line 29 veterinary, breeding and medicine"),
    ("DISPLAY_4835_LINE30_MEALS", "Line 30 business meals, before the 50% limit"),
    ("DISPLAY_4835_LINE5A_PRICE_ONLY", "Line 5a payments based only on price or a rainfall index"),
    ("DISPLAY_4835_LINE5C_DEFERRAL", "Line 5c deferral to 2027"),
    ("DISPLAY_4835_LINE5D_FROM_PRIOR_YEAR", "Line 5d amount deferred from 2025"),
    ("DISPLAY_FARM_INCOME", "Income, lines 4–6"),
    ("DISPLAY_FARM_COSTS", "Costs"),
    ("DISPLAY_FARM_RENTAL_INCOME", "Income, lines 1–6"),
    ("DISPLAY_FARM_RENTAL_MORE", "More about this rental"),
    ("DISPLAY_LAND_SALES", "Land sold"),
    ("DISPLAY_TAX_ELECTIONS", "Tax and elections"),
    (
        "NOTE_CCC_SECTION_77_TREATMENT_RETAIN_THE_LOAN_DETAIL_ELECTION_STATEME_BA3452BD",
        "CCC section 77 treatment: retain the loan-detail election statement and individual commodity basis records. The election applies to all current and later loans unless an authorized method change applies; the App does not file it.",
    ),
    (
        "NOTE_CROP_INSURANCE_DEFERRAL_SELECTED_ATTACH_THE_REQUIRED_SECTION_451_42E57F41",
        "Crop-insurance deferral selected: attach the required section 451(f)/Regulations 1.451-6 statement and retain the 2027 income amount. The election covers all eligible proceeds of this farming business and is binding unless the IRS approves a change. The App does not file the statement.",
    ),
    (
        "NOTE_SCHEDULE_J_CHANGES_REGULAR_INCOME_TAX_ONLY_RETAIN_THE_DISPLAYED_29693665",
        "Schedule J changes regular income tax only. Retain the displayed base-year amounts for later elections; AGI, self-employment tax, QBI, carryovers and AMT are unchanged. Final 2026 form reconciliation remains required.",
    ),
    (
        "NOTE_THE_SELECTED_FARM_AVERAGING_ELECTION_INCREASES_REGULAR_INCOME_TA_9202791E",
        "The selected farm-averaging election increases regular income tax. Reduce the chosen amount or remove the election to use ordinary current-year tax; no alternative is elected automatically.",
    ),
    (
        "ISSUE_AN_ACCRUAL_SECTION_77_ELECTION_NEEDS_COORDINATION_BETWEEN_THE_FARM_4B04A5BF",
        "An accrual section 77 election needs coordination between the farm's inventory valuation and the loan's income-inclusion and redemption basis. Aggregate opening/closing inventory alone does not establish that adjustment. Keep the recorded election; do not change an established method to bypass this unfinished worksheet.",
    ),
    (
        "ISSUE_AN_ASSESSMENT_FIRST_SPREAD_IN_THE_YEAR_OF_DEATH_REQUIRES_SEPARATE_F_18A72DF0",
        "An assessment first spread in the year of death requires separate final-year treatment; the statutory acceleration covers death during the nine succeeding years.",
    ),
    (
        "ISSUE_AN_EXISTING_INCOME_ELECTION_CONTINUES_FOR_CURRENT_LOANS_CHANGING_TO_BA0302A2",
        "An existing income election continues for current loans; changing to loan treatment requires an accounting-change worksheet.",
    ),
    (
        "ISSUE_A_FARM_LAND_DISPOSITION_WITH_HOME_OFFICE_USE_REQUIRES_GAIN_LOSS_ATT_317F08BC",
        "A farm-land disposition with home-office use requires gain/loss attribution to the home's business-income limit before coordinating the supply ceiling. Operating receipts alone do not establish that limit.",
    ),
    (
        "ISSUE_A_SALE_AFTER_THIS_FARM_OWNER_S_DEATH_DOES_NOT_BELONG_TO_THE_OWNER_S_E146F4F3",
        "A sale after this farm owner's death does not belong to the owner's final individual return. Resolve the estate or beneficiary's ownership and reporting; a surviving spouse's own farm follows that spouse's dates.",
    ),
    (
        "ISSUE_A_SUPPORTED_DEFICIENCY_MUST_BE_REQUIRED_AND_ACTUALLY_PAID_FOR_A_SEC_708807CC",
        "A supported deficiency must be required and actually paid for a section 77 income-included loan, and cannot exceed principal. Loan-method deficiencies need a separate worksheet; released or unpaid shortfalls are not deductions.",
    ),
    (
        "ISSUE_CONFIRM_ONE_CONSISTENTLY_ADOPTED_ELIGIBLE_POULTRY_COST_METHOD_AN_UN_25613C7F",
        "Confirm one consistently adopted eligible poultry-cost method. An uncertain, income-distorting or changed method needs its own accounting worksheet; neither the supply exception nor this editor elects a new method.",
    ),
    (
        "ISSUE_CONFIRM_TEN_YEARS_OF_QUALIFYING_USE_A_QUALIFIED_INDIVIDUAL_BUYER_AN_E0DF1DF7",
        "Confirm ten years of qualifying use, a qualified individual buyer, an enforceable ten-year covenant and a recognized gain before electing section 1062.",
    ),
    (
        "ISSUE_ENTER_THREE_PRIOR_YEARS_EXACTLY_ONCE_11CCA3B9",
        "Enter 2023, 2024 and 2025 exactly once.",
    ),
    (
        "ISSUE_FARM_PRODUCT_BASIS_AND_INVENTORY_ARE_ALREADY_ACCOUNTED_FOR_IN_THE_F_63EC9604",
        "Farm product basis and inventory are already accounted for in the farm records; do not also deduct Schedule C inventory costs.",
    ),
    (
        "ISSUE_OTHER_TAXES_CANNOT_EXCEED_THE_RETAINED_FORM_1040_LINE_16_AMOUNT_A3DD421C",
        "Other taxes cannot exceed the retained Form 1040 line 16 amount.",
    ),
    (
        "ISSUE_PRE_2018_FARM_EQUIPMENT_REQUIRES_ITS_APPLICABLE_150_OR_STRAIGHT_LIN_D2ED857C",
        "Pre-2018 farm equipment requires its applicable 150% or straight-line depreciation method, not ordinary 200% declining balance.",
    ),
    (
        "ISSUE_PRIOR_CASH_METHOD_CROP_DEFERRALS_ON_AN_ACCRUAL_METHOD_RETURN_REQUIR_B6A28F1E",
        "Prior cash-method crop deferrals on an accrual-method return require an accounting-change worksheet.",
    ),
    (
        "ISSUE_RECONCILE_THE_BASE_YEAR_CAPITAL_LOSS_DEDUCTION_AND_CARRYOVER_THE_UN_C2A79762",
        "Reconcile the base-year capital-loss deduction and carryover; the unused deduction must be between zero and the deduction claimed.",
    ),
    (
        "ISSUE_REDEMPTION_PRINCIPAL_CANNOT_EXCEED_THE_ORIGINAL_PRINCIPAL_REDEMPTIO_7D5465D5",
        "Redemption principal cannot exceed the original principal; redemption or forfeiture cannot precede a current-year loan. Enter interest and fees separately.",
    ),
    (
        "ISSUE_RETAIN_ONLY_ONE_FORM_2555_PER_BASE_YEAR_FILER_A_SPOUSE_S_FORM_BELON_B6664F34",
        "Retain only one Form 2555 per base-year filer; a spouse's form belongs only on that year's joint return.",
    ),
    (
        "ISSUE_SCHEDULE_F_LINE_4B_THE_SECTION_126_EXCLUSION_FOR_A_CONSERVATION_COS_9EAC6444",
        "Schedule F line 4b: the section 126 exclusion for a conservation cost-sharing payment (the present-value computation of Pub 225 chapter 3 and its attached statement) is not calculated. Enter a payment that is fully taxable as an agricultural program payment instead.",
    ),
    (
        "ISSUE_SUPPLY_A_POSITIVE_ELECTION_AND_ALL_THREE_VALID_BASE_YEAR_RECORDS_82AB2094",
        "Supply a positive election and all three valid base-year records.",
    ),
    (
        "ISSUE_SUPPLY_NEGATIVE_INCOME_RECORDS_EXACTLY_WHEN_THE_BASE_YEAR_S_AGI_MIN_DF6BD01E",
        "Supply negative-income records exactly when the base year's AGI minus deductions is negative.",
    ),
    (
        "ISSUE_THE_ELECTED_AMOUNT_MUST_BE_POSITIVE_AND_CANNOT_EXCEED_ELIGIBLE_FARM_DF8EA144",
        "The elected amount must be positive and cannot exceed eligible farm income after attributable deductions or current taxable income.",
    ),
    (
        "ISSUE_THE_PREPAID_CEILING_STILL_NEEDS_COORDINATION_WHEN_SEVERAL_FARMS_COM_960C7361",
        "The prepaid ceiling needs its dependent expense worksheet when separate-return owners or accounting methods differ, section 179 or separate AMT depreciation applies, or source records are incomplete.",
    ),
    (
        "ISSUE_THE_RETAINED_FORM_4952_DIVIDEND_ELECTION_EXCEEDS_THE_BASE_YEAR_QUAL_047D93F4",
        "The retained Form 4952 dividend election exceeds the base-year qualified dividends; reconcile lines 4g and 4e, including any dotted-line allocation.",
    ),
    (
        "ISSUE_THE_RETAINED_NEGATIVE_INCOME_ADJUSTMENTS_EXCEED_THE_BASE_YEAR_DEFIC_3CA2371E",
        "The retained negative-income adjustments exceed the base-year deficit; reconcile the NOL and capital-loss worksheets.",
    ),
    (
        "ISSUE_THIS_ELECTION_STILL_NEEDS_COORDINATION_WITH_FULL_FORM_1116_PARENT_R_44C75BFA",
        "This election still needs coordination with parent-rate tax or a final return. No installment totals are shown until the applicable worksheet is supported.",
    ),
    (
        "ISSUE_THIS_PREPAID_SUPPLY_WORKSHEET_USES_CASH_METHOD_TIMING_ACCRUAL_SUPPL_B035D15E",
        "This prepaid-supply worksheet uses cash-method timing. Accrual supply expenses follow their separate recognition records.",
    ),
    (
        "ISSUE_TWO_FARMING_SPOUSES_WITH_SEPARATE_BASE_YEAR_RETURNS_REQUIRE_SEPARAT_45269592",
        "Two farming spouses with separate base-year returns require separate histories and an election allocation. A sole farmer may use differing actual filing statuses.",
    ),
    (
        "ISSUE_USE_THE_MOST_RECENT_RETAINED_SCHEDULE_J_FOR_EVERY_BASE_YEAR_THAT_IT_1C243B9E",
        "Use the most recent retained Schedule J for every base year that it adjusts, and the original/corrected return for later base years.",
    ),
    (
        "ISSUE_USE_THE_SAME_CURRENT_YEAR_CCC_METHOD_FOR_EVERY_FARM_OWNED_BY_THIS_T_C9BBF517",
        "Use the same current-year CCC method for every farm owned by this taxpayer.",
    ),
    (
        "PATR_WITHHOLDING_ABOVE_DISTRIBUTIONS",
        "Box 4 cannot exceed the distributions in boxes 1, 2, 3 and 5.",
    ),
    (
        "PATR_QUALIFIED_ITEMS_ABOVE_DISTRIBUTIONS",
        "Box 8 cannot exceed the distributions in boxes 1, 2, 3 and 5.",
    ),
    (
        "FARM_COST_SHARING_NEEDS_CASH",
        "Enter excluded cost-sharing payments only on a cash-method farm.",
    ),
    (
        "CROP_PAPER_ALLOCATION_INVALID",
        "The physical-loss portions must match the printed proceeds: all proceeds for several physical crops, or a smaller physical-loss total when the remainder is nondeferrable.",
    ),
    (
        "CROP_PAPER_FACTS_NEEDED",
        "Crop-insurance proceeds are included in current income. Open this activity’s crop-insurance details to answer its missing timing or physical-loss facts; only an established election can defer income.",
    ),
    (
        "CROP_PRINTED_PAYMENTS_NEED_RECORDS",
        "Complete this activity's crop-insurance payment records to include the amounts on its related Forms 1099-MISC. Include each payment once and use the insurer and farm records to identify physical-loss and other portions.",
    ),
    (
        "CROP_WITHHOLDING_ABOVE_PAYMENTS",
        "Withholding from crop insurance cannot exceed the payments plus the amount deferred from 2025.",
    ),
    ("CCC_REPAID_ABOVE_PRINCIPAL", "The principal you repaid cannot exceed the loan principal."),
    ("CCC_DEFICIENCY_ABOVE_PRINCIPAL", "The deficiency you paid cannot exceed the loan principal."),
    ("CONSERVATION_SHARE_ABOVE_TOTAL", "Your share cannot exceed the district's total assessment."),
    (
        "CONSERVATION_PAYMENTS_ABOVE_SHARE",
        "An assessment's payments cannot add up to more than your share.",
    ),
    ("FARM_LAND_SOLD_BEFORE_ACQUIRED", "The date sold cannot be before the date acquired."),
    ("FARM_LAND_SOLD_OUTSIDE_CURRENT_YEAR", "Enter only land sold in 2026."),
    (
        "FARM_WITHHOLDING_ABOVE_RECEIPTS",
        "Federal income tax withheld cannot exceed the farm's income.",
    ),
    (
        "AVERAGING_NEGATIVE_BASE_NEEDS_LOSS",
        "When the deductions are more than the adjusted gross income, enter the negative taxable-income records.",
    ),
    (
        "AVERAGING_SOURCE_BEFORE_BASE_YEAR",
        "A later Schedule J cannot be from a year before the base year it refigures.",
    ),
    ("PRINT_YES", "Yes"),
    ("PRINT_NO", "No"),
    ("PRINT_NOT_APPLICABLE", "Not applicable"),
    (
        "FARM_LEASE_ACTUAL_NOT_SUPPORTED",
        "A farm vehicle lease using actual expenses needs the Schedule F rental and vehicle worksheets. Use supported standard mileage for the entire lease or remove this vehicle record.",
    ),
    (
        "FARM_RENTAL_NO_INVESTMENT_CREDIT",
        "The investment credit for the property of a farm rental is not calculated.",
    ),
    ("DISPLAY_F_LINES_11_18", "Lines 11–18"),
    ("DISPLAY_F_LINES_19_23", "Lines 19–23"),
    ("DISPLAY_F_LINES_24_28", "Lines 24–28"),
    ("DISPLAY_F_LINES_29_32", "Lines 29–32"),
];
