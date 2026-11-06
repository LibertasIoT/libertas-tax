pub(crate) const HOME_STRINGS: &[(&str, &str)] = &[
    ("MORTGAGE_PAID_INTEREST_NEEDED", "Enter the interest you paid from your payment records."),
    (
        "MORTGAGE_PAID_INTEREST_ABOVE_BOX1",
        "Your paid share cannot exceed the interest printed in box 1.",
    ),
    ("MORTGAGE_DRAW_DATE_NEEDED", "Enter the date this credit-line money was borrowed."),
    ("MORTGAGE_POINTS_TREATMENT_NEEDED", "Choose how the points printed in box 6 were used."),
    ("MORTGAGE_INSURANCE_CONTRACT_DATE_NEEDED", "Enter the mortgage insurance contract date."),
    (
        "HOME_SITE_IDENTITY_INVALID",
        "Save each physical home once with its stable accepted identity. A duplicate or missing home identity cannot establish source links.",
    ),
    (
        "HOME_BUILDING_SOURCE_INVALID",
        "Correct the building's dates, nonnegative basis and value, land portions, or retained depreciation history. Earlier simplified years must be unique completed years after business use began.",
    ),
    (
        "HOME_ACTIVITY_ALLOCATION_INVALID",
        "Enter each activity once, use shares from zero through 10000 and positive allocation parts, and explain any partial income or expense-period share.",
    ),
    (
        "HOME_ACTIVITY_PERIOD_SHARES_EXCESS",
        "This activity's shares across its homes exceed its annual income or expenses. Allocate the actual annual source only once.",
    ),
    (
        "HOME_ACTIVITY_REQUIRED",
        "Select at least one saved business or farm for this home-use claim.",
    ),
    (
        "HOME_ACTIVITY_SOURCE_INVALID",
        "Select a saved business or farm belonging to the owner of this home-use claim.",
    ),
    (
        "HOME_ACTIVITY_SOURCE_UNAVAILABLE",
        "A linked activity is unavailable in this calculation. Resolve its source records before claiming the dependent home-use amounts.",
    ),
    (
        "HOME_AT_RISK_LOSS_SOURCE_UNAVAILABLE",
        "The home-use deduction creates a business loss whose at-risk source records are unavailable. No completed tax is reported from an assumed risk amount.",
    ),
    (
        "HOME_BUSINESS_AREA_ABOVE_HOME",
        "Enter positive qualifying business space whose combined exclusive and shared areas do not exceed the home area.",
    ),
    (
        "HOME_CARRY_NEGATIVE",
        "Carryovers cannot be negative. Enter the retained operating, depreciation and casualty amounts from their supporting worksheets.",
    ),
    (
        "HOME_CASUALTY_CHARACTER_UNAVAILABLE",
        "The allowed casualty amount or its retained character is unavailable. Resolve the linked casualty worksheet.",
    ),
    (
        "HOME_CASUALTY_DAMAGE_REQUIRED",
        "A property linked to a business home needs its actual damage status, including an explicit unavailable-records answer when necessary. An unlinked property leaves that answer inactive.",
    ),
    (
        "HOME_CASUALTY_DAMAGE_SOURCE_UNAVAILABLE",
        "The records do not establish whether this business property was damaged, destroyed or stolen. Its business casualty amount remains unavailable.",
    ),
    (
        "HOME_GAIN_ATTRIBUTION_SOURCE_UNAVAILABLE",
        "The accepted sale records do not identify the capital or business-property gain attributable to this home. The home-use income limit remains unavailable; the sale source is retained.",
    ),
    (
        "HOME_CASUALTY_GAIN_CHARACTER_UNAVAILABLE",
        "This business-property reimbursement produces a gain whose recognition, recapture and replacement-election worksheet is not yet available.",
    ),
    (
        "HOME_CASUALTY_HOME_BINDING_INVALID",
        "Bind this casualty property to one accepted home and select the same event on that home. Conflicting property bindings cannot establish a deduction.",
    ),
    (
        "HOME_CASUALTY_OUTPUT_UNAVAILABLE",
        "The linked casualty amounts cannot be represented by the native worksheet. Correct the actual source records before calculating.",
    ),
    (
        "HOME_CASUALTY_PROPERTY_LINK_REQUIRED",
        "Each selected home casualty event must contain exactly one property explicitly bound to that home.",
    ),
    (
        "HOME_CASUALTY_SOURCE_UNAVAILABLE",
        "The linked casualty source is incomplete. Its business-use loss cannot be inferred from the personal amount or from unavailable records.",
    ),
    (
        "HOME_COMBINED_USE_ABOVE_HOME",
        "The owners' combined allocated business use exceeds the whole property. Allocate their actual space once.",
    ),
    (
        "HOME_COST_OUTSIDE_USE_PERIOD",
        "Use a cost period within each affected owner's qualifying home-use period, and select an owner with a claim on this home.",
    ),
    (
        "HOME_COST_PERIOD_AMOUNT_INVALID",
        "Enter a nonnegative home cost and valid dates in chronological order.",
    ),
    (
        "HOME_DAYCARE_TIME_INVALID",
        "Daycare hours cannot exceed the calendar hours available for shared use. Enter zero shared-use hours when no shared area is claimed.",
    ),
    (
        "HOME_DEPRECIATION_HISTORY_UNAVAILABLE",
        "This home depreciation schedule needs its retained historical-method, disposition or production-property worksheet. No replacement depreciation amount is assumed.",
    ),
    ("HOME_LAND_ABOVE_BASIS", "The land basis cannot exceed the whole-property adjusted basis."),
    ("HOME_LAND_ABOVE_VALUE", "The land value cannot exceed the whole-property conversion value."),
    (
        "HOME_MORTGAGE_CREDIT_SOURCE_LINK_UNAVAILABLE",
        "The mortgage interest credit needs an explicit link between its certificate and this home's canonical loan before the home-use interest can be completed.",
    ),
    (
        "HOME_MORTGAGE_ROOT_TWICE",
        "Select each original mortgage loan once and bind it to one home. Later lenders follow that same original loan.",
    ),
    ("HOME_MORTGAGE_SOURCE_INVALID", "Select a saved original mortgage loan for this home."),
    (
        "HOME_OWNER_CLAIM_REQUIRED",
        "Add one claim for each owner using this home, with no more than one claim per owner.",
    ),
    (
        "HOME_OWNER_CLAIM_TWICE",
        "Combine this owner's activities into one home-use claim for this physical home.",
    ),
    (
        "HOME_PAYMENT_SHARE_UNAVAILABLE",
        "Enter positive parts for both owners when dividing their paid home costs.",
    ),
    ("HOME_SAVED_HOME", "Home {0}: {1}"),
    ("HOME_SAVED_CASUALTY_EVENT", "Casualty event {0}: {1}"),
    (
        "HOME_SHARED_AREA_NEEDS_DAYCARE",
        "Shared business area requires an established daycare use. Other home-use qualifications require the qualifying exclusive area.",
    ),
    (
        "HOME_SIMPLIFIED_AREA_ABOVE_HOME",
        "The monthly exclusive and shared qualifying areas cannot exceed the whole home area.",
    ),
    (
        "HOME_SIMPLIFIED_DAYS_INVALID",
        "The qualifying-day count cannot exceed the actual calendar days in that month.",
    ),
    (
        "HOME_SIMPLIFIED_ONE_HOME_PER_OWNER",
        "Each owner can elect the simplified method for at most one physical home during the year. Another home uses its actual-expense worksheet.",
    ),
    (
        "HOME_SIMPLIFIED_SHARED_NEEDS_DAYCARE",
        "Simplified shared-use area and days require a daycare use.",
    ),
    (
        "HOME_SIMPLIFIED_TWELVE_MONTHS",
        "Enter each of the twelve months exactly once, using zero qualifying days for a month without qualifying use.",
    ),
    (
        "HOME_SITE_ADDRESS_AREA_REQUIRED",
        "Enter an address or identifying name for this home and its positive total area.",
    ),
    (
        "HOME_USE_DATES_REVERSED",
        "Enter valid first and last qualifying-use dates within 2026, in chronological order.",
    ),
    (
        "HOME_USE_FRACTION_UNAVAILABLE",
        "The home area, allocated business space and daycare time do not establish a valid business-use fraction.",
    ),
    (
        "NOTE_HOME_CASUALTY_PROPORTIONAL_REDUCTIONS",
        "Shared casualty reductions were allocated proportionally by each property's remaining net loss, with cent rounding in original property order.",
    ),
    (
        "WARN_HOME_USE_NOT_ESTABLISHED",
        "Qualifying business use of this home was not established; its requested home-use deduction was left out.",
    ),
    ("HOME_SAVED_ACTIVITY", "{1} (business or farm {0})"),
    (
        "RENTAL_CASUALTY_SALVAGE_UNAVAILABLE",
        "This destroyed property source does not establish its actual remaining salvage value. Its casualty loss remains unavailable.",
    ),
    (
        "RENTAL_CASUALTY_CREDIT_UNAVAILABLE",
        "This property's tax credit needs its coordinated casualty, credit, recapture and basis worksheets before the complete result is available.",
    ),
    (
        "RENTAL_CASUALTY_GAIN_UNAVAILABLE",
        "This rental casualty gain needs the return's Section B gain netting, depreciation recapture and final tax-character worksheets.",
    ),
    (
        "RENTAL_CASUALTY_NONRECURRING_UNAVAILABLE",
        "This exceptional casualty loss needs its separate passive-activity exclusion and coordinated at-risk worksheet. It cannot be treated as a recurring rental loss.",
    ),
    (
        "RENTAL_CASUALTY_CONTINUATION_UNAVAILABLE",
        "This surviving rental property needs its dated adjusted-basis depreciation schedule after the casualty. Its original cost and credit records cannot substitute for that schedule.",
    ),
    (
        "RENTAL_CASUALTY_DISPOSITION_UNAVAILABLE",
        "This property has both a casualty and another disposition. Reconcile their actual dates, remaining basis and depreciation before estimating.",
    ),
    (
        "RENTAL_CASUALTY_EVENT_YEAR_UNAVAILABLE",
        "This earlier insurance-resolution event or next-year disaster election needs its actual event-year basis and election worksheet. Those results remain unavailable.",
    ),
    (
        "RENTAL_CASUALTY_ELECTION_DECLARATION",
        "A next-year disaster elected onto this return requires a federal disaster declaration.",
    ),
    (
        "RENTAL_CASUALTY_PROPERTY_BINDING_UNAVAILABLE",
        "On Rentals, link each damaged building or improvement to its saved whole-dwelling casualty property. The linked costs must represent distinct parts of that dwelling, with consistent use and actual basis, value and reimbursement records. A property cannot be assigned to competing dwellings or business-home uses.",
    ),
    ("RENTAL_CASUALTY_EVENT_REQUIRED", "Name the casualty or theft event."),
    (
        "RENTAL_CASUALTY_DATE",
        "Enter a valid date. The casualty or theft-discovery date cannot be later than the year after this return.",
    ),
    (
        "RENTAL_CASUALTY_DECLARATION",
        "Use the actual declaration reference; choose the major-disaster option only for a presidential DR- declaration.",
    ),
    (
        "RENTAL_CASUALTY_AMOUNT",
        "Casualty property values, basis, depreciation, reimbursements and replacement costs cannot be negative.",
    ),
    (
        "RENTAL_CASUALTY_DEPRECIATION_RECORDS",
        "Prior-year depreciation cannot exceed depreciation through the event. The event-date adjusted basis plus that depreciation cannot exceed the original depreciable basis.",
    ),
    (
        "RENTAL_CASUALTY_BASIS_HISTORY_UNAVAILABLE",
        "This rental casualty needs actual event-date basis and depreciation history. Current personal-use days and sale-date records cannot establish that history.",
    ),
    (
        "RENTAL_CASUALTY_HOLDING_PERIOD_UNAVAILABLE",
        "This casualty needs the actual acquisition holding-period records for the converted or gifted property.",
    ),
    (
        "RENTAL_CASUALTY_OUTPUT_UNAVAILABLE",
        "The casualty records do not establish a representable property loss, gain or remaining basis.",
    ),
    (
        "MORTGAGE_REFUND_ORIGIN_NEEDED",
        "Answer what the Form 1098 box 4 refund repays on the mortgage questions page.",
    ),
    (
        "MORTGAGE_REFUND_ORIGIN_ANSWERED",
        "{0,plural,one {# refund origin is} other {# refund origins are}} answered; change {0,plural,one {it} other {them}} here",
    ),
    (
        "MORTGAGE_PURCHASE_POINTS",
        "Purchase points require a loan used to buy your main home. Choose the points treatment that fits this loan.",
    ),
    (
        "MORTGAGE_IMPROVEMENT_POINTS",
        "Immediate improvement points require your main home and loan proceeds used to improve it.",
    ),
    (
        "MORTGAGE_EARLIER_LOAN_AFTER_THIS_ONE",
        "The earlier loan cannot begin after the Form 1098 box 3 date of this loan.",
    ),
    (
        "MORTGAGE_EARLIER_TERM_NEEDED",
        "Enter the original loan term end date to apply its grandfathered or earlier acquisition-debt treatment.",
    ),
    (
        "MORTGAGE_BALANCE_ABOVE_AMOUNTS_BORROWED",
        "Box 2 cannot exceed the total borrowed for the earlier balance, improvements and other uses.",
    ),
    (
        "MORTGAGE_MIXED_USE_NEEDS_MONTHLY",
        "For mixed-use proceeds, choose month-end balances, first and last balances with level payments, or the confirmed within-limit method.",
    ),
    ("MORTGAGE_MONTH_TWICE", "Enter each month-end balance only once."),
    (
        "MORTGAGE_EARLIER_LENDER_ENTRY_NEEDED",
        "Enter the earlier lender with the balances for the whole year before marking another form as the same loan.",
    ),
    (
        "MORTGAGE_LOAN_ID_INVALID",
        "Reopen Home mortgage interest. Each saved lender entry must keep its own loan identity.",
    ),
    (
        "MORTGAGE_LOAN_SELF_REFERENCE",
        "Choose the saved first lender's entry, not this entry itself.",
    ),
    (
        "MORTGAGE_LOAN_ORIGINATION_DIFFERS",
        "These entries must have the same printed box 3 origination date to identify one transferred loan.",
    ),
    (
        "MORTGAGE_RENTAL_ROOT_NEEDED",
        "A rental still uses this loan. Remove its loan selection on Rentals before deleting or replacing the first lender's entry.",
    ),
    (
        "WARN_HOME_SALE_DEPRECIATION_LEFT_OUT",
        "{0}: depreciation other than straight-line (accelerated, bonus or Section 179) on a home isn't calculated (Form 4797), so the sale is left out of this estimate; your tax may be higher.",
    ),
    (
        "WARN_HOME_SALE_OTHER_LEFT_OUT",
        "{0}: this special home-sale situation isn't calculated, so the sale is left out of this estimate; your tax may be higher.",
    ),
    (
        "NOTE_STUDENT_LOAN_ENTRY_ID_REV_PROC_2020_11_RELIEF_ALSO_PREVE_F831B367",
        "Student-loan entry {0,number,0}: Rev. Proc. 2020-11 relief also prevents discharge-triggered recapture of prior education credits, student-loan interest and tuition deductions. Do not report that recapture solely because of this discharge. Separate cash refunds or other recoveries need their own treatment; retain the final discharge or settlement and eligible-school records.",
    ),
    (
        "ISSUE_HOME_SALE_SEPARATE_DEPRECIATION_UNKNOWN",
        "The depreciation on the separate business or rental space is not known, so its gain and the share of the price left for your home cannot be figured (Form 4797; Pub. 523). Your Schedule C, Schedule E and Form 4562 records show it.",
    ),
    (
        "ISSUE_HOME_SALE_SEPARATE_SHORT_TERM",
        "A separate business or rental space sold within a year of acquisition is ordinary income on Form 4797 Part II, which is not calculated.",
    ),
    (
        "ISSUE_HOME_SALE_SEPARATE_PASSIVE_LOSS",
        "Passive rental losses carried over from earlier years are released by the sale of the rental space (Form 8582, section 469(g)), which is not calculated here.",
    ),
    (
        "ISSUE_HOME_SALE_PRIOR_EXCHANGE_UNKNOWN",
        "Whether the earlier owner got this home in a like-kind exchange within 5 years of the sale is not known, so the exclusion cannot be told (section 121(d)(10), Pub. 523).",
    ),
    (
        "ISSUE_HOME_SALE_PROPERTY_VALUE_UNKNOWN",
        "The value of the property or services received as part of the price (1099-S box 6) is not known, so the amount realized cannot be figured (Pub. 523).",
    ),
    (
        "ISSUE_HOME_SALE_FIRPTA_WITHHOLDING_UNKNOWN",
        "The tax the buyer withheld from a foreign seller (Form 8288-A) is not known, so the credit on Form 1040 line 25c cannot be figured.",
    ),
    (
        "ISSUE_HOME_SALE_INSTALLMENT_SEPARATE",
        "An installment sale that also sells a separate business or rental space is not calculated (a separate Form 6252 for each part).",
    ),
    (
        "ISSUE_HOME_SALE_INSTALLMENT_CONTRACT_PRICE",
        "An installment sale needs a contract price above zero: the loans the buyer took over must be less than the sale price (Form 6252 line 18).",
    ),
    (
        "ISSUE_HOME_SALE_EXCHANGE_COMBINED",
        "An exchange of a home is calculated for a whole property held more than a year and paid in full; a separate business or rental space, an installment note or a short holding period is not calculated with it (Form 8824).",
    ),
    (
        "ISSUE_HOME_SALE_SPECIAL_2",
        "Sale of your main home: the tax on a person who gave up citizenship or long-term residence (Form 8854, section 877A) is a return-wide tax on a dual-status return, which is not calculated.",
    ),
    (
        "ISSUE_HOME_SALE_PROVIDE_BOTH_SPOUSES__HOME_HISTORIES_FOR_325E7C69",
        "Provide both spouses' home histories for a joint return and only the taxpayer's otherwise.",
    ),
    (
        "ISSUE_HOME_SALE_PROVIDE_SEPARATE_REDUCED_EXCLUSION_PERIODS_FOR_EACH_2AB2C8F5",
        "Provide separate reduced-exclusion periods for each joint filer, and no spouse periods on other returns.",
    ),
    (
        "ISSUE_HOME_SALE_RESOLVE_HOME_SALE_BASIS__NONQUALIFIED_USE__DEPRECIATION_84E3D92F",
        "Depreciation on a home held one year or less is not calculated (Form 4797).",
    ),
    (
        "ISSUE_HOME_SALE_RESOLVE_THE_SURVIVING_SPOUSE_HOME_SALE_HISTORY__REMARRIAGE_D8822A28",
        "The surviving-spouse exclusion needs your own two-year ownership and residence, no excluded sale in the two years before and a sale held more than a year; otherwise evaluate a reduced exclusion.",
    ),
    (
        "ISSUE_HOME_SALE_THE_FULL_OWNERSHIP_RESIDENCE_EXCLUSION_IS_NOT_A9D4EF05",
        "The full ownership/residence exclusion is not established; evaluate reduced or special exclusions before completing this sale.",
    ),
    (
        "ISSUE_HOME_SALE_THE_SELECTED_EVENT_DOES_NOT_ESTABLISH_BB23B05D",
        "The selected event does not establish a reduced home-sale exclusion; review other facts and circumstances.",
    ),
    (
        "ISSUE_DEBT_JOINT_CARRYOVER_SHARE",
        "A joint return needs each spouse's share of the carryover this canceled debt reduces (Treas. Reg. 1.108-7(d), Form 982 line {0,number,integer}); it is not calculated.",
    ),
    (
        "ISSUE_DEBT_EXCESS_BUSINESS_LOSS",
        "An excess business loss carryforward (Form 461 line 16) reached by canceled debt (Form 982 line 6) is not calculated.",
    ),
    (
        "ISSUE_DEBT_MINIMUM_TAX_CREDIT_UNKNOWN",
        "The minimum tax credit from 2026 alternative minimum tax (Form 8801 for 2027) that canceled debt reduces (Form 982 line 8) is not calculated.",
    ),
    (
        "ISSUE_DEBT_GBC_CARRYBACK",
        "A general business credit carried back to 2025 that canceled debt reduces (Form 982 line 7) is not calculated.",
    ),
    (
        "ISSUE_DEBT_MTC_CARRYBACK",
        "A minimum tax credit carried back to 2025 that canceled debt reduces (Form 982 line 8) is not calculated.",
    ),
    (
        "ISSUE_DEBT_PARTIAL_MIXED_CAPITAL",
        "Canceled debt that reduces only part of a mixed short- and long-term capital loss carryover (Form 982 line 9) needs the year-of-origin ledger; it is not calculated.",
    ),
    (
        "ISSUE_DEBT_BUSINESS_PROPERTY_MISSING",
        "List the business, farm and rental property you hold on January 1, 2027 under Property, with its basis: canceled debt reduces it before personal-use property (Form 982 line 10a).",
    ),
    (
        "ISSUE_DEBT_FTC_CARRYBACK",
        "A foreign tax credit carried back to 2025 that canceled debt reduces (Form 982 line 13) is not calculated.",
    ),
    (
        "ISSUE_DEBT_MULTIPLE_DISCHARGES",
        "Several discharges with exclusions for one filer in 2026 need the multiple-discharge basis allocation (Treas. Reg. 1.1017-1(b)(2), Form 982 Part II); it is not calculated.",
    ),
    (
        "ISSUE_DEBT_FACTS_UNCONFIRMED",
        "Confirm the discharge facts, including a liability or principal snapshot that includes the canceled debt, and the property held on January 1, 2027, after completing the 2026 return.",
    ),
    (
        "ISSUE_DEBT_FARM_LIMIT_EXCEEDED",
        "Canceled farm debt above the Form 982 line 1c limit (adjusted tax attributes plus the basis of qualified property, {0,number,currency}) is Schedule F line 8 income, which is not calculated.",
    ),
    (
        "ISSUE_DEBT_FARM_LIMIT_MTC_UNKNOWN",
        "Canceled farm debt above the Form 982 line 1c limit (adjusted tax attributes plus the basis of qualified property, {0}) is Schedule F line 8 income, which is not calculated. The limit also needs the minimum tax credit from 2026 alternative minimum tax (Form 8801 for 2027).",
    ),
    (
        "ISSUE_DEBT_REAL_PROPERTY_LIMIT_EXCEEDED",
        "Canceled real property business debt above the Form 982 line 1d limit ({0,number,currency}) is Schedule C line 6, Schedule E line 3 or Schedule F line 8 income, which is not calculated.",
    ),
    (
        "ISSUE_DEBT_REAL_PROPERTY_LIST_MISSING",
        "List the depreciable real property that secured the debt, and your other buildings, under Property with their January 1, 2027 basis (Form 982 line 4).",
    ),
    (
        "ISSUE_CASUALTY_MIXED_TERM_EVENT_REDUCTION",
        "An excess-gain Form 4684 event with short- and long-term property losses needs allocation of its single $100 or $500 event reduction.",
    ),
    (
        "ISSUE_CASUALTY_MIXED_TERM_ROUNDING",
        "Mixed-term personal casualty gains need a Form 4684 whole-dollar allocation worksheet.",
    ),
    ("DISPLAY_HOME_1099S", "1099-S"),
    ("DISPLAY_HOME_FACTS", "Your home"),
    ("DISPLAY_8828_LOAN", "Loan and dates"),
    ("DISPLAY_LIVING_EVENT", "The loss of use"),
    ("DISPLAY_LIVING_PAYMENTS", "Payments and expenses"),
    ("DISPLAY_LIVING_TAX", "Earlier returns and disaster area"),
    ("DISPLAY_8828_NOTICE", "Recapture notice"),
    (
        "ISSUE_DEBT_BUSINESS_DEBT",
        "Canceled business debt included in income (Schedule C line 6, Schedule E line 3, Schedule F line 8 or Form 4835 line 6) is not calculated.",
    ),
    (
        "ISSUE_DEBT_PRINCIPAL_RESIDENCE",
        "Qualified principal residence debt canceled under a written arrangement made before 2026 (Form 982 lines 1e and 10b) is not calculated.",
    ),
    (
        "ISSUE_DEBT_LIQUIDATION_BANKRUPTCY",
        "Debt discharged in a chapter 7 or chapter 11 case, with its separate bankruptcy estate (Form 982 line 1a), is not calculated.",
    ),
    (
        "ISSUE_DEBT_DEALER_PROPERTY_ELECTION",
        "The election to treat real property held for sale to customers as depreciable property (Form 982 line 3) is not calculated.",
    ),
    (
        "ISSUE_DEBT_OTHER",
        "This canceled-debt situation (Pub. 4681) is not calculated; the canceled amount is not treated as tax-free.",
    ),
    (
        "NOTE_CCC_FORFEITURE_ALREADY_REPORTED",
        "Form 1099-A entry {0}: this forfeited CCC loan was reported as income under the section 77 election, so the forfeiture is not counted again; your basis in the commodity equals the {1,number,currency} reported (Schedule F lines 5a-5c, Pub. 225 chapter 3).",
    ),
    (
        "NOTE_CCC_FORFEITURE_IS_THE_SALE",
        "Form 1099-A entry {0}: you did not elect to report this CCC loan as income, so the forfeiture is the sale and the {1,number,currency} is income (Schedule F lines 5b and 5c, Pub. 225 chapter 3); it is counted once, from the CCC loan record.",
    ),
    (
        "NOTE_CCC_FORFEITURE_ALREADY_REPORTED_RENTAL",
        "Form 1099-A entry {0}: this forfeited CCC loan was reported as income under the section 77 election, so the forfeiture is not counted again; your basis in the commodity equals the {1,number,currency} reported (Form 4835 lines 4a-4c, Pub. 225 chapter 3).",
    ),
    (
        "NOTE_CCC_FORFEITURE_IS_THE_SALE_RENTAL",
        "Form 1099-A entry {0}: you did not elect to report this CCC loan as income, so the forfeiture is the sale and the {1,number,currency} is income (Form 4835 lines 4b and 4c, Pub. 225 chapter 3); it is counted once, from the CCC loan record.",
    ),
    (
        "NOTE_RECOURSE_DEBT_ABOVE_PROPERTY_VALUE",
        "Form 1099-A entry {0}: the {1,number,currency} of recourse debt above the property's value is not income until the lender cancels it; enter that lender's Form 1099-C when it does.",
    ),
    (
        "ISSUE_TRANSFERRED_PROPERTY_SALE_PRICE",
        "Form 4797: enter this business, farm or rental property's 2026 sale on its own page with {0,number,currency} (the amount realized) as the sale price.",
    ),
    // The running estimate's warning of what saving the page removes or
    // reopens (walk-owned data).
    (
        "ISSUE_ABANDONED_PROPERTY_THE_LENDER_HAS_NOT_TAKEN_A_PARTIAL_TRANSFER_OR_M_1D9AC0DD",
        "Abandoned property the lender has not taken, a partial transfer or mixed-use property (Pub. 4681 chapters 2 and 3; Form 4797 or Form 8949) is not calculated.",
    ),
    (
        "ISSUE_A_MAIN_HOME_FORECLOSURE_WITH_A_GAIN_NEEDS_THE_HOME_SALE_EXCLUSION_S_02E3DE73",
        "A main home foreclosure with a gain is entered on the Sale of your main home page, with the amount realized shown here as its price, so the home-sale exclusion applies (section 121; Pub. 4681).",
    ),
    (
        "ISSUE_CONFIRM_THE_COMPLETED_QUALIFYING_DISCHARGE_AND_LOAN_CATEGORY_DEATH_E03BC664",
        "Death/disability relief in 2026 requires this filer's employment-authorized SSN issued by the return deadline and included on the return. Missing qualification needs the applicable taxable-debt or other exclusion worksheet; do not assume relief.",
    ),
    (
        "ISSUE_CONFIRM_THE_ORIGINAL_SELLER_S_REDUCTION_OF_PURCHASE_PRINCIPAL_SOLVE_7D3164A4",
        "Confirm the original seller's reduction of purchase principal, solvency, no bankruptcy or debt transfer, adequate property basis and retained personal ownership or the disclosed ordinary sale. Other cases need their cancellation or disposition worksheet.",
    ),
    (
        "ISSUE_FORM_1099_C_BOX_5_IS_NOT_CHECKED_A_NONRECOURSE_DEBT_CANCELED_WITH_I_7768A375",
        "Form 1099-C box 5 is not checked: a nonrecourse debt canceled with its property is part of the amount realized, not canceled-debt income. Enter the property on a Form 1099-A entry with the whole debt as box 2. A nonrecourse debt reduced without a transfer (Pub. 4681) is not calculated.",
    ),
    (
        "ISSUE_A_FORM_1099_A_FOR_A_CROP_MUST_NAME_A_CCC_LOAN_FORFEITED_IN_CURRENT_YEAR_WITH_THE_SAME_PRINCIPAL",
        "A Form 1099-A for a crop forfeited to the Commodity Credit Corporation must name a loan of your farm or farm rental that was forfeited in 2026 and whose principal equals box 2.",
    ),
    (
        "ISSUE_THE_PART_OF_A_BUSINESS_FARM_OR_RENTAL_DEBT_ABOVE_YOUR_INSOLVENCY_IS_CFEE5ABB",
        "The part of a business, farm or rental debt above your insolvency is Schedule C line 6, Schedule E line 3, Schedule F line 8 or Form 4835 line 6 income, which is not calculated.",
    ),
    (
        "MESSAGE_FORM_8915_F_YOUR_SPOUSE_S_FORMS_GO_ON_THIS_RETURN_ONLY_WHEN_YOU_FILE_JOINTLY",
        "Form 8915-F: your spouse's forms go on this return only when you file jointly.",
    ),
    (
        "HEALTH_LOAN_INTEREST_ABOVE_ASSISTANCE",
        "The interest paid through the program cannot exceed the assistance.",
    ),
    ("RELIEF_RECEIVED_OUTSIDE_2026", "Enter a payment received in 2026."),
    (
        "WILDFIRE_DECLARED_OUTSIDE_2015_2026",
        "The fire's federal disaster declaration must be dated from January 1, 2015 through December 31, 2026.",
    ),
    ("RELIEF_WITHHOLDING_ABOVE_PAYMENT", "Withholding cannot exceed the payment."),
    (
        "DISASTER_CARRIER_ACCIDENT_NEEDS_CARRIER_PAYMENT",
        "An accident involving a common carrier supports only a payment from the carrier; choose that purpose.",
    ),
    (
        "DISASTER_ASSISTANCE_NEEDS_GOVERNMENT_PAYMENT",
        "Government assistance supports only a government welfare payment; choose that purpose.",
    ),
    (
        "CASUALTY_LIVING_REGAINED_BEFORE_LOST",
        "The date you regained use cannot be before the date you lost it.",
    ),
    (
        "CASUALTY_LIVING_DISASTER_NEEDS_REFERENCE",
        "Enter the FEMA declaration reference for a federally declared disaster area.",
    ),
    (
        "CASUALTY_LIVING_REFERENCE_WITHOUT_DISASTER",
        "Leave the FEMA declaration reference empty unless the casualty was in a federally declared disaster area.",
    ),
    (
        "DEBT_LIABILITIES_BELOW_CANCELED",
        "Liabilities immediately before the cancellation include the canceled debt: enter at least box 2.",
    ),
    (
        "DEBT_PRICE_REDUCTION_ABOVE_BASIS",
        "A seller's price reduction cannot exceed the property's adjusted basis.",
    ),
    (
        "DEBT_PRICE_REDUCTION_ABOVE_LIABILITIES",
        "Liabilities immediately before the reduction include the seller debt: enter at least box 2.",
    ),
    (
        "DEBT_SALE_BEFORE_REDUCTION",
        "The later sale cannot come before the price reduction in box 1.",
    ),
    (
        "DEBT_PRINCIPAL_BELOW_CANCELED",
        "The outstanding principal immediately before the cancellation includes the canceled amount: enter at least box 2.",
    ),
    (
        "DEBT_ELECTION_ABOVE_CANCELED",
        "The amount you elect to apply first to depreciable property cannot exceed box 2.",
    ),
    (
        "DISASTER_ELECTION_DIFFERS",
        "Choose the same timing for every qualified disaster distribution of disasters that began in the same year (Form 8915-F lines 11 and 22).",
    ),
    (
        "DISASTER_EARLIER_DIFFERS",
        "Enter the same earlier qualified disaster distributions on every 1099-R for one disaster (Form 8915-F line 1b).",
    ),
    (
        "FORM_8915_TWICE",
        "Enter each earlier Form 8915-F once for each filer, tax year and disaster year.",
    ),
    (
        "HOME_ACQUIRED_AFTER_SALE",
        "Sale of your main home: the date you got the home must be before the closing date (1099-S box 1).",
    ),
    (
        "HOME_SALE_NOT_IN_CURRENT_YEAR",
        "Sale of your main home: enter a sale that closed in 2026 (1099-S box 1).",
    ),
    (
        "HOME_LIVED_ABOVE_OWNED",
        "Sale of your main home: the days lived there cannot exceed the days since you got the home, or the 1,826 days of the 5 years before the sale.",
    ),
    (
        "HOME_SEPARATE_AREA_ABOVE_WHOLE",
        "Sale of your main home: the separate space must be smaller than the whole property, in the same unit.",
    ),
    (
        "HOME_SEPARATE_DEPRECIATION_ABOVE_COST",
        "Sale of your main home: the depreciation on the separate space cannot exceed its share of the cost, which is your adjusted basis plus that depreciation, by floor area.",
    ),
    (
        "HOME_SERVICE_SPOUSE_NEEDED",
        "Sale of your main home: on a joint return, enter the days your spouse lived there in the added period (0 if none).",
    ),
    (
        "HOME_SERVICE_SPOUSE_NOT_JOINT",
        "Sale of your main home: leave your spouse's days in the added period empty unless you file a joint return.",
    ),
    (
        "HOME_SERVICE_WITH_SURVIVING",
        "Sale of your main home: the service election is not combined with the surviving-spouse exclusion on this page.",
    ),
    (
        "HOME_SERVICE_LIVED_ABOVE_OWNED",
        "Sale of your main home: the days lived there in the added period cannot exceed the days you owned the home before the 5 years, up to the days of service elected.",
    ),
    (
        "HOME_PRIOR_OWNER_AFTER_ACQUIRED",
        "Sale of your main home: the earlier owner's date cannot be after the date you got the home.",
    ),
    (
        "HOME_PROPERTY_VALUE_WITHOUT_BOX_6",
        "Sale of your main home: enter the value of property or services received only when 1099-S box 6 is checked.",
    ),
    (
        "HOME_FIRPTA_WITHOUT_BOX_7",
        "Sale of your main home: enter the tax the buyer withheld only when 1099-S box 7 says you are a foreign person.",
    ),
    (
        "HOME_EXCHANGE_VALUE_ABOVE_PRICE",
        "Sale of your main home: the like-kind property received cannot be worth more than the selling price.",
    ),
    (
        "HOME_EARLIER_SALE_AFTER_SALE",
        "Sale of your main home: the earlier home sale must be before this closing date.",
    ),
    (
        "HOME_NONQUALIFIED_ABOVE_OWNED",
        "Sale of your main home: the days nobody used it as a main home cannot exceed the days you owned it.",
    ),
    (
        "HOME_DEATH_AFTER_SALE",
        "Sale of your main home: your spouse's date of death must be on or before the closing date.",
    ),
    (
        "HOME_SURVIVING_WITH_REASON",
        "Sale of your main home: the surviving spouse exclusion needs the 2-year tests; for a move for work, health or an unforeseen event, choose Nothing else under Something else.",
    ),
    (
        "HOME_SURVIVING_ON_JOINT",
        "Sale of your main home: on a joint return the spouse's own facts apply; the surviving spouse choice is for a return filed after the death year.",
    ),
    (
        "HOME_SPOUSE_NEEDED",
        "Sale of your main home: on a joint return, answer your spouse's facts under Your spouse.",
    ),
    (
        "HOME_SPOUSE_NOT_JOINT",
        "Sale of your main home: your spouse's facts are asked only on a joint return.",
    ),
    (
        "MORTGAGE_LAST_BALANCE_ABOVE_BOX_2",
        "The year-end balance cannot exceed the outstanding principal at the start of the year (box 2).",
    ),
    (
        "DISASTER_SOURCE_TEXT_INVALID",
        "Enter a nonblank carrier, disaster reference or agency name.",
    ),
    ("DISPLAY_REISSUE_ORIGINAL", "Original certificate"),
    ("DISPLAY_REISSUE_RATE", "Rate and certified debt"),
];
