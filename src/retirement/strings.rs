pub(crate) const RETIREMENT_STRINGS: &[(&str, &str)] = &[
    (
        "R_UNDETERMINED_METHOD_NEEDED",
        "Box 2b says the pension taxable amount was not determined. Select the applicable basis-recovery worksheet.",
    ),
    ("RRB_RECIPIENT_NEEDED", "Choose whose railroad annuity this is."),
    ("RRB_SOURCE_NONNEGATIVE", "Enter a nonnegative amount for this railroad statement box."),
    (
        "RRB_RECOVERY_COST_NEEDED",
        "Box 3 is blank on this statement, or the annuity started before 2026. Enter the employee-contribution cost for this recovery worksheet, or answer No to recovering the box 3 contributions.",
    ),
    (
        "RRB_ANNUITANT_BIRTH_NEEDED",
        "Your age at the annuity starting date comes from your date of birth, which is missing or later than that date. Correct it on the About you page, or choose the entered age.",
    ),
    (
        "R_ANNUITANT_BIRTH_NEEDED",
        "Your age at the annuity starting date comes from your date of birth, which is missing or later than that date. Correct it on the About you page, or choose the entered age.",
    ),
    (
        "SOCIAL_PREMIUM_ALLOCATION_CONFLICT",
        "The separate allocations must fit this statement’s Medicare premiums, the selected plan’s existing premiums and any qualified HSA spending already entered. Review the changed amounts.",
    ),
    (
        "OWNER_PLAN_DEFERRAL_ABOVE_LIMIT",
        "Enter elective deferrals and catch-up contributions within the 2026 plan limit; the App applies your age and shared W-2 limits.",
    ),
    (
        "OWNER_PLAN_ROTH_ABOVE_DEFERRALS",
        "The designated Roth amount cannot exceed deferrals plus catch-up contributions.",
    ),
    (
        "NOTE_OWNER_PLAN_LIMITED",
        "The owner retirement deduction was limited by compensation, the plan formula, age or shared elective deferrals. Arrange any excess correction with the plan; Form 5330 excise tax is not calculated.",
    ),
    (
        "NOTE_DEFINED_BENEFIT_ACTUARY",
        "The defined benefit deduction uses the actuary's deductible amount, limited to this business's net earnings. The actuarial limit is not calculated.",
    ),
    (
        "INHERITED_ROTH_FIRST_YEAR_AFTER_DEATH",
        "The first Roth IRA year cannot be after the year the person died.",
    ),
    (
        "WARN_ROTH_HISTORY_ASSUMED_NONE",
        "Assumed your Roth IRA started in 2026 with no earlier contributions or conversions, because its history wasn't entered, so a withdrawal that isn't qualified is taxed as earnings; if you have an earlier Roth IRA or basis, your tax may be lower.",
    ),
    (
        "WARN_ROTH_BASIS_ASSUMED_ZERO",
        "Assumed no earlier Roth IRA contributions or conversions, because their records weren't entered, so a withdrawal that isn't qualified is taxed as earnings; if you have that basis, your tax may be lower.",
    ),
    (
        "ISSUE_LUMP_SUM_TREATED_DIFFERENTLY",
        "All of a participant's lump-sum distributions in one year must be taxed the same way. Choose the same Form 4972 method for each 1099-R of this participant, or tax none of them separately.",
    ),
    (
        "ISSUE_LUMP_SUM_SHARED_AND_COMBINED",
        "A lump sum shared with other recipients (box 9a) or with an annuity share (box 8b) is figured from its own statement. It cannot be combined with another 1099-R of the same participant's lump sum.",
    ),
    (
        "AFTER_DEATH_ABOVE_BOX",
        "A part paid after the death can't be more than the box it comes from.",
    ),
    ("RRB_STATEMENT_YEAR", "Enter the RRB-1099 for this return’s tax year."),
    ("RRB_EARLIER_YEARS_DIFFER", "The attached earlier-year amounts must total RRB-1099 box 9."),
    (
        "RRB_EARLIER_YEAR_NEEDED",
        "The attached box 9 breakdown must name a year before the years in boxes 7 and 8.",
    ),
    (
        "WARN_PENSION_CONTRIBUTIONS_LEFT_OUT",
        "Section 501(c)(18)(D) contributions that aren't confirmed aren't supported, so their deduction is left out of this estimate; your tax may be lower.",
    ),
    (
        "ISSUE_8915_MISSING_FEMA_DECLARATION",
        "Form 8915-F item C: only a disaster with a FEMA major disaster declaration (a DR number) can have qualified disaster distributions. Choose No, or enter the DR number.",
    ),
    (
        "ISSUE_PENSION_TRADITIONAL_IRA_LIMIT",
        "Traditional IRA contributions exceed the remaining annual or compensation limit after section 501(c)(18)(D) contributions; resolve the excess before calculating this return.",
    ),
    (
        "ISSUE_IRA_NEXT_YEAR_DEPOSITS",
        "Confirm every actual 2027 IRA deposit was made by April 15 and designated for 2026 with its custodian. Its traditional and Roth amounts must be included once in the matching retained 2026 contribution totals.",
    ),
    (
        "ISSUE_IRA_NONDEDUCTIBLE_ELECTION",
        "The voluntary nondeductible amount must be between zero and this filer's regular traditional contributions.",
    ),
    (
        "ISSUE_IRA_SPOUSE_COVERAGE",
        "Complete both IRA coverage records for a joint return; remove spouse IRA facts for any other filing status.",
    ),
    ("DISPLAY_LUMP_SUM_RETURN", "That year's return"),
    ("DISPLAY_INHERITED_ACCOUNT", "Account"),
    ("DISPLAY_INHERITED_ELECTION", "Eligibility"),
    ("DISPLAY_4972_PARTICIPANT", "Participant and amounts"),
    ("DISPLAY_4972_ELECTION", "Tax method"),
    ("DISPLAY_4972_CONDITIONS", "Payments and conditions"),
    ("DISPLAY_SALE_DISTRIBUTION", "Plan distribution"),
    ("DISPLAY_ANNUITY_CONTRACT", "The contract"),
    ("DISPLAY_ANNUITY_PAYMENTS", "Payments"),
    ("DISPLAY_ANNUITY_RECORDS", "Records"),
    ("DISPLAY_VARIABLE_CONTRACT", "The contract"),
    ("DISPLAY_VARIABLE_PAYMENTS", "Payments"),
    ("DISPLAY_VARIABLE_RECORDS", "Records"),
    (
        "ISSUE_IRA_TRADITIONAL_EXCESS_NEEDS_YEAR_END_VALUES",
        "Form 5329 Part III line 15: 2026 traditional IRA contributions exceed this filer's age-based or compensation limit. Enter the December 31, 2026 IRA values under More about these contributions on the IRA contributions page so the 6% tax is calculated, or correct the excess.",
    ),
    (
        "ISSUE_IRA_ROTH_EXCESS_NEEDS_YEAR_END_VALUES",
        "Form 5329 Part IV line 23: 2026 Roth IRA contributions exceed this filer's income-based, age-based or compensation limit. Enter the December 31, 2026 IRA values under More about these contributions on the IRA contributions page so the 6% tax is calculated, or correct or recharacterize the excess.",
    ),
    ("INHERITED_IRA_CHOICE", "{0} ({1,date}) — traditional IRA, {2}"),
    ("INHERITED_ROTH_CHOICE", "{0} ({1,date}) — Roth IRA, {2}"),
    (
        "ISSUE_1099R_CODE_5",
        "Form 1099-R code 5 from a Roth IRA (prohibited transaction) is not calculated yet.",
    ),
    (
        "ISSUE_1099R_CODE_C",
        "Form 1099-R code C (reportable death benefits, section 6050Y) is not calculated yet.",
    ),
    (
        "ISSUE_1099R_CODE_F",
        "Form 1099-R code F (charitable gift annuity) with a box 3 capital gain is not calculated yet.",
    ),
    (
        "ISSUE_1099R_CODES_PAIR",
        "Form 1099-R codes {0} and {1} printed together in box 7a are not calculated yet.",
    ),
    ("ISSUE_1099R_CODE_OTHER", "Form 1099-R box 7a code {0} is not calculated yet."),
    (
        "ISSUE_DISASTER_LIMIT_TAXPAYER",
        "Line 1e: your qualified disaster distributions for disaster DR-{0} cannot exceed $22,000 less earlier years; treat less as qualified.",
    ),
    (
        "ISSUE_DISASTER_LIMIT_SPOUSE",
        "Line 1e: your spouse's qualified disaster distributions for disaster DR-{0} cannot exceed $22,000 less earlier years; treat less as qualified.",
    ),
    (
        "ISSUE_AN_ELIGIBLE_BENEFICIARY_CANNOT_ELECT_THE_BEFORE_BEGINNING_DATE_FIXE_4059242D",
        "An eligible beneficiary cannot elect the before-beginning-date fixed payout rule after the owner reached the required beginning date.",
    ),
    (
        "ISSUE_AN_EXPIRED_OR_CURRENT_FINAL_PAYOUT_DEADLINE_NEEDS_ADMINISTRATOR_REV_FF8034CB",
        "An expired or current final payout deadline needs administrator review of the remaining balance and prior shortfalls; it cannot be approximated with an annual life-expectancy fraction.",
    ),
    (
        "ISSUE_AN_IRA_TO_EMPLOYER_PLAN_ROLLOVER_CAN_CONTAIN_ONLY_PRETAX_FUNDS_THE_EA576D4E",
        "An IRA-to-employer-plan rollover can contain only pretax funds. The retained IRA balances and amounts not rolled into that plan must cover this owner's available after-tax basis.",
    ),
    (
        "ISSUE_ENTER_ONE_ROTH_IRA_HISTORY_PER_OWNER_817D4C12",
        "Enter one Roth IRA history per owner (on the Retirement topic): the first Roth IRA year and the contributions and conversions not yet taken out (Form 8606 lines 22 and 24).",
    ),
    (
        "ISSUE_A_1099_R_WITH_CODE_T_FOR_AGE_59_IS_DATED_BEFORE_THE_OWNER_TURNED_59_3DA81978",
        "A 1099-R with code T for age 59½ is dated before the owner turned 59½: check the distribution date and birth date, or choose disability as the reason.",
    ),
    (
        "ISSUE_A_2026_QUALIFIED_ROTH_TO_HSA_FUNDING_TRANSFER_TOGETHER_WITH_529_TO_AFFC98A6",
        "A 2026 qualified Roth-to-HSA funding transfer together with 529-to-Roth transfers needs separate source-date and basis accounting.",
    ),
    (
        "ISSUE_A_FORM_8915_F_FROM_2023_OR_EARLIER_HAS_NO_2026_INCOME_A_2026_REPAYM_8F28E5B6",
        "A Form 8915-F from 2023 or earlier has no 2026 income; a 2026 repayment of those distributions is carried back to 2023-2025 on amended returns (Form 1040-X, Form 8915-F lines 14 and 25), which this App does not prepare.",
    ),
    (
        "ISSUE_A_PARTIAL_EXCEPTION_NEEDS_A_TRADITIONAL_IRA_ALLOCATION_THE_QUALIFIE_C205A7E3",
        "A partial exception needs a traditional IRA allocation, the qualified pre-annuity withdrawal worksheet, or a complete qualified employer-plan discharge. Other non-IRA basis methods and overlapping public-safety premium exclusions need source-specific allocation.",
    ),
    (
        "ISSUE_A_REQUIREMENT_WHOSE_DEADLINE_FOLLOWS_THE_OWNER_S_DEATH_NEEDS_ADMINI_80CF0FDD",
        "A requirement whose deadline follows the owner's death needs administrator allocation between the decedent and beneficiaries; do not charge the deceased filer's return for another payee's shortfall.",
    ),
    (
        "ISSUE_A_SOLE_SPOUSE_S_MISSED_INHERITED_IRA_MINIMUM_CAN_TRIGGER_DEEMED_OWN_C6B1E9C1",
        "A sole spouse's missed inherited IRA minimum can trigger deemed ownership. Reconcile the election and complete personal IRA pool with the administrator before calculating a shortfall; do not impose the nonspouse inherited shortfall mechanically.",
    ),
    (
        "ISSUE_A_WAIVER_REQUEST_NEEDS_A_REASONABLE_ERROR_EXPLANATION_AND_REMEDIAL_AC7AD5EB",
        "A waiver request needs a reasonable-error explanation and remedial steps; its amount cannot exceed the actual shortfall. IRS approval remains pending.",
    ),
    (
        "ISSUE_CLAIMS_FOR_ONE_PARENT_AND_CHILD_MUST_USE_THE_SAME_BIRTH_ADOPTION_DA_F05B51E0",
        "Claims for one parent and child must use the same birth/adoption date and opening prior-distribution total.",
    ),
    (
        "ISSUE_COMBINED_MEDICAL_EXPENSE_AND_UNEMPLOYMENT_INSURANCE_EXCEPTIONS_NEED_05A37B66",
        "Combined medical-expense and unemployment-insurance exceptions need allocation of overlapping premiums before calculation.",
    ),
    (
        "ISSUE_CONFIRM_ACTUAL_ACCEPTED_REPAYMENTS_AND_USE_ONE_RECORD_PER_ORIGINAL_6D4178D6",
        "Use one repayment record per original Income entry.",
    ),
    (
        "ISSUE_INHERITED_IRA_OTHER_OWNER",
        "This payment names an Inherited IRA that belongs to the other filer. Choose one of this filer's own.",
    ),
    (
        "ISSUE_INHERITED_IRA_KIND_MISMATCH",
        "This payment names an Inherited IRA of the other kind: a Roth IRA payment needs the Roth record and a traditional IRA payment the traditional one.",
    ),
    (
        "ISSUE_CONFIRM_A_LIVING_ORIGINAL_DESIGNATED_BENEFICIARY_AND_OWNER_DEATH_BE_764F8CE9",
        "Confirm a living original designated beneficiary and owner death before 2026. Successor/trust rules, other spouse elections and unresolved source allocations need the administrator's minimum.",
    ),
    (
        "ISSUE_CONFIRM_A_PARTICIPATING_QUALIFIED_DEFINED_CONTRIBUTION_PLAN_AND_THE_442EED49",
        "Confirm a participating qualified defined-contribution plan and the issuer's certified annual premium statement filed before payment. IRA and ordinary retroactively designated withdrawals do not qualify.",
    ),
    (
        "ISSUE_CONFIRM_A_QUALIFIED_DISTRIBUTION_DURING_ELIGIBLE_RESERVE_DUTY_AN_OR_0A51B128",
        "Confirm a qualified distribution during eligible reserve duty, an order for at least 180 days or indefinitely, and actual traditional IRA repayments after release and within two years. All repayments together cannot exceed that original distribution.",
    ),
    (
        "ISSUE_CONFIRM_COMPLETE_OWN_IRA_BALANCES_AND_SOURCE_FACTS_USE_THE_INHERITE_5B1D22CE",
        "Confirm complete own-IRA balances and source facts; use the inherited branch for supported beneficiary accounts or an administrator's minimum for annuity and unresolved special-relief cases.",
    ),
    (
        "ISSUE_CONFIRM_ORDINARY_OWN_ACCOUNT_WITHDRAWALS_AND_SOURCE_DATES_INHERITED_68CEF704",
        "Confirm ordinary own-account withdrawals and source dates. Inherited, special repayment, rollover, unentered HSA funding and unlisted exception cases require further support.",
    ),
    (
        "ISSUE_CONFIRM_ORDINARY_OWN_ACCOUNT_WITHDRAWALS_FOR_AN_OWNER_PRESENT_ON_TH_44ECD1D8",
        "Confirm ordinary own-account withdrawals for an owner present on this return.",
    ),
    (
        "ISSUE_CONFIRM_PAID_FEDERAL_ESTATE_TAX_THE_ESTATE_WIDE_RECOMPUTATION_AND_G_2E262284",
        "Confirm paid federal estate tax, the estate-wide recomputation and gross IRD denominator, and this beneficiary's unused allocation. A shared QCD cap or deductible-contribution adjustment across several sources needs an explicit source allocation before this deduction can be determined.",
    ),
    (
        "ISSUE_CONFIRM_THE_DISTRIBUTION_YEAR_S_MARRIAGE_AND_SOLE_BENEFICIARY_DESIG_1F368E99",
        "Confirm the distribution year's marriage and sole-beneficiary designation, including changes after divorce or death.",
    ),
    (
        "ISSUE_CONFIRM_THE_ELIGIBLE_ACCOUNT_OR_ANNUITY_SOURCE_DATES_AND_EXCEPTION_1865F21C",
        "Confirm the eligible account or annuity, source dates and exception requirements; Roth recapture and unentered repayment adjustments need another worksheet.",
    ),
    (
        "ISSUE_CONFIRM_THE_SOURCE_MINIMUM_TIMELY_PAYMENTS_LEGAL_ACCOUNT_GROUP_AND_2AAF74CB",
        "Confirm the source minimum, timely payments, legal account group and filer. Unresolved deadlines or source calculations require review.",
    ),
    (
        "ISSUE_CONTRIBUTIONS_EXCEED_THE_7_000_25_COMPENSATION_OR_2026_SHARED_24_50_BB098953",
        "Contributions exceed the $7,000, 25% compensation or 2026 shared $24,500 regular-deferral limit. Excess-contribution and catch-up corrections need resolution before this return can be calculated.",
    ),
    (
        "ISSUE_DOMESTIC_ABUSE_CLAIMS_FOR_ONE_OWNER_MUST_USE_THE_SAME_PRE_2026_QUAL_77D1C43F",
        "Domestic-abuse claims for one owner must use the same pre-2026 qualified gross total.",
    ),
    (
        "ISSUE_DOMESTIC_ABUSE_CLAIMS_FROM_ONE_PLAN_MUST_USE_THE_SAME_OPENING_BALAN_956F1ABF",
        "Domestic-abuse claims from one plan must use the same opening balance and pre-2026 plan usage.",
    ),
    (
        "ISSUE_EACH_TRUSTEE_PAYMENT_MUST_BE_MADE_ON_OR_AFTER_THE_RECEIVING_FILER_S_17F2FD62",
        "Each trustee payment must be made on or after the receiving filer's actual age-70½ date, including gifts from inherited IRAs.",
    ),
    (
        "ISSUE_EARLIER_QUALIFIED_AMOUNTS_ASSIGNED_TO_CURRENT_PLANS_CANNOT_EXCEED_T_7D6D3EC7",
        "Earlier qualified amounts assigned to current plans cannot exceed the owner's total earlier qualified amount.",
    ),
    (
        "ISSUE_ENTER_EACH_EARLIER_FORM_8915_F_ONCE_WITH_A_DISASTER_YEAR_NO_LATER_T_087EE17B",
        "Enter each earlier Form 8915-F once, with a disaster year no later than its tax year and a carried-back amount no larger than its line 14 or 25 excess.",
    ),
    (
        "ISSUE_ENTER_ONE_CONFIRMED_ANNUAL_SECTION_501_C_18_D_RECORD_PER_FILER_SPOU_D34001ED",
        "Enter one confirmed annual section 501(c)(18)(D) record per filer; spouse contributions require a joint return.",
    ),
    (
        "ISSUE_ESTABLISH_K_1_BOX_14_CODE_A_BEFORE_PLAN_COMPENSATION_C9E84E0E",
        "Establish K-1 box 14 code A before plan compensation.",
    ),
    (
        "ISSUE_EVERY_IRA_BASIS_SELECTION_NEEDS_A_COMPLETE_SUPPORTED_OWNER_LEVEL_WO_C967CBA5",
        "Every IRA-basis selection needs a complete supported owner-level worksheet in Adjustments; it cannot determine pension or annuity taxation.",
    ),
    (
        "ISSUE_FIRST_HOME_CLAIMS_FOR_ONE_IRA_OWNER_MUST_USE_THE_SAME_PRE_2026_LIFE_BBE3640F",
        "First-home claims for one IRA owner must use the same pre-2026 lifetime total.",
    ),
    (
        "ISSUE_FIRST_YEAR_DEFERRAL_IS_AVAILABLE_ONLY_FOR_THE_YEAR_THIS_OWNER_TURNS_FC1B39D7",
        "First-year deferral is available only for the year this owner turns 73; later IRA requirements cannot be deferred for continued employment.",
    ),
    (
        "ISSUE_FORM_8915_F_LINES_11_AND_22_ONE_ELECTION_APPLIES_TO_EVERY_QUALIFIED_BC403371",
        "Form 8915-F lines 11 and 22: one election applies to every qualified disaster distribution of the same disaster year on this filer's form. Choose the same timing on each.",
    ),
    (
        "ISSUE_FORM_8915_F_LINE_1B_ENTER_THE_SAME_EARLIER_QUALIFIED_DISASTER_DISTR_530EA91B",
        "Form 8915-F line 1b: enter the same earlier qualified disaster distributions on every 1099-R for this disaster.",
    ),
    (
        "ISSUE_INHERITED_GIFTS_MUST_FOLLOW_THE_ORIGINAL_OWNER_S_DEATH_AND_BE_INCLU_434B63F8",
        "Inherited gifts must follow the original owner's death and be included in the linked group's gross beneficiary distributions.",
    ),
    (
        "ISSUE_K_1_BOX_13_CODE_R_FOR_A_SIMPLE_IRA_401_K_DEFINED_BENEFIT_PLAN_OR_IR_F5178D19",
        "K-1 box 13 code R for an IRA payment or a SEP with other facts (Schedule 1 line 16 or 20; Form 7206 line 9) is not calculated.",
    ),
    (
        "ISSUE_LIFE_EXPECTANCY_IS_EXHAUSTED_SUPPLY_THE_ACTUAL_DECEMBER_31_2026_REM_23509D0F",
        "Life expectancy is exhausted: supply the actual December 31, 2026 remaining account value, including earnings. Last year's balance is not the full-distribution shortfall.",
    ),
    (
        "ISSUE_LINK_A_SAME_OWNER_2026_TRADITIONAL_IRA_BIRTH_ADOPTION_EMERGENCY_DOM_D69B7407",
        "Link a same-owner 2026 traditional IRA birth/adoption, emergency, domestic-abuse or terminal-illness distribution. Repay after distribution and within three years, no more than its qualified gross amount.",
    ),
    (
        "ISSUE_LINK_EACH_INHERITED_GIFT_TO_AN_EXISTING_INHERITED_TRADITIONAL_IRA_I_4140777A",
        "Link each inherited gift to an existing inherited traditional IRA income entry belonging to this filer.",
    ),
    (
        "ISSUE_LINK_EACH_PREMIUM_TO_THE_OFFICER_S_GOVERNMENTAL_PENSION_OR_DISABILI_42B94848",
        "Link each premium to the officer's governmental pension or disability-pension Income entry with a supported taxable-amount calculation. An IRA, commercial annuity or railroad pension does not qualify.",
    ),
    (
        "ISSUE_MULTIPLE_SELF_EMPLOYED_PLANS_NEED_COMBINED_PLAN_LIMIT_AND_RELATED_E_661C9DAF",
        "Multiple self-employed plans need combined plan-limit and related-employer worksheets.",
    ),
    (
        "ISSUE_ONLY_ONE_ANNUAL_EMERGENCY_DISTRIBUTION_PER_PERSON_IS_PERMITTED_ENTE_4A862D65",
        "Only one annual emergency distribution per person is permitted; enter traditional IRA repayments in the owner's basis record.",
    ),
    (
        "ISSUE_ORDINARY_ROTH_WITHDRAWALS_BEFORE_THE_LAST_QUALIFIED_FUNDING_TRANSFE_E42B2BF2",
        "Ordinary Roth withdrawals before the last qualified funding transfer require supported own-account nonqualified, dated ordering. Current regular contributions need enough recorded earnings to cover every funding transfer without using basis; other earlier activity needs further support. Funding-only earlier entries are permitted.",
    ),
    (
        "ISSUE_PREMIUMS_ALLOCATED_TO_ONE_PENSION_CANNOT_EXCEED_ITS_AVAILABLE_DISTR_9D7632D9",
        "Premiums allocated to one pension cannot exceed its available distribution after rollovers.",
    ),
    (
        "ISSUE_PROVIDE_THIS_RECIPIENT_S_IRA_BASIS_RECORD_AFTER_AN_EMPLOYER_PLAN_RO_E8156654",
        "Provide this recipient's IRA basis record after an employer-plan rollover of after-tax contributions. The App adds the incoming basis automatically; do not include it in prior basis.",
    ),
    (
        "ISSUE_REPEAT_ONE_PLAN_S_ANNUAL_VALUATION_CONSISTENTLY_ACROSS_ITS_PAYMENTS_FB3D8571",
        "Repeat one plan's annual valuation consistently across its payments.",
    ),
    (
        "ISSUE_RESOLVE_ACTUAL_PARTNERSHIP_SE_EARNINGS_BEFORE_CALCULATING_PLAN_COMP_84586D00",
        "Resolve actual partnership SE earnings before calculating plan compensation.",
    ),
    (
        "ISSUE_SCHEDULE_K_1_1065_BOX_13_CODE_R_REPORTS_A_RETIREMENT_PLAN_PAYMENT_F_39BE1646",
        "Schedule K-1 (1065) box 13 code R reports a retirement plan payment for you: choose its plan for this partnership under self-employed health insurance and retirement plan (Schedule 1 line 16; Form 7206 line 9).",
    ),
    (
        "ISSUE_SPLIT_MIXED_QUALIFYING_AND_NONQUALIFYING_SCHEDULE_R_DISABILITY_PAYM_8A5A7A30",
        "Split mixed qualifying and nonqualifying Schedule R disability payments before allocating the insurance exclusion.",
    ),
    (
        "ISSUE_THE_EXCEPTION_S_DATE_ACCOUNT_TYPE_EMPLOYMENT_ELIGIBILITY_OR_SEPARAT_9E1325F3",
        "The exception's date, account type, employment eligibility or separation age/service does not qualify for this distribution.",
    ),
    (
        "ISSUE_THE_FIRST_ROTH_IRA_YEAR_IN_THE_ROTH_IRA_HISTORY_CANNOT_BE_BEFORE_TH_8D09C080",
        "The first Roth IRA year in the Roth IRA history cannot be before the owner's birth year.",
    ),
    (
        "ISSUE_THE_SEP_CONTRIBUTION_EXCEEDS_THE_2026_INCOME_BASED_PLAN_RATE_COMPEN_A4C70725",
        "The SEP contribution exceeds the 2026 income-based, plan-rate compensation or $72,000 annual limit; excess correction or carryover needs a separate worksheet.",
    ),
    (
        "ISSUE_THE_SUPPORTED_INHERITED_ACCOUNT_UNIFORM_LIFETIME_ELECTION_REQUIRES_3A658F85",
        "The supported inherited-account Uniform Lifetime election requires a sole surviving spouse age 73 or older in 2026 and an original owner who died in 2023 or later, on or after the required beginning date. Reconcile other election histories with the administrator.",
    ),
    (
        "ISSUE_THIS_SEP_WORKSHEET_REQUIRES_ONE_SELF_EMPLOYMENT_BUSINESS_FOR_THIS_O_0FB340F4",
        "The owner plan worksheet requires one self-employment business and one self-employed plan for this owner.",
    ),
    (
        "ISSUE_USE_ONE_COMPLETE_BASIS_RECORD_PER_FILER_AND_SELECT_THE_IRA_WORKSHEE_F3EC6BF8",
        "Use one complete basis record per filer and select the IRA worksheet for all their traditional IRA distributions. Contributions paid after 2026 must include every actual 2027 traditional deposit designated for 2026 and cannot exceed retained traditional contributions. Inherited accounts use their separate deceased-owner record; changed basis requires additional coordination. Exception eligibility and any gross-distribution limits are checked separately.",
    ),
    (
        "ISSUE_USE_ONE_COMPLETE_QCD_RECORD_PER_FILER_RECONCILE_GIFTS_TO_ENTERED_IR_167E808F",
        "Use one QCD record per filer and reconcile gifts to entered IRA distributions. Personal after-tax sources need the owner-level basis worksheet; inherited basis stays in its linked income entry.",
    ),
    (
        "ISSUE_USE_ONE_CONSISTENT_ANNUAL_CERTIFIED_PREMIUM_TOTAL_FOR_EACH_COVERED_255AFF79",
        "Use one consistent annual certified premium total for each covered person, not a fresh allowance per payment.",
    ),
    (
        "ISSUE_USE_ONE_RECORD_PER_ORIGINAL_DISTRIBUTION_AND_COMBINE_ITS_2026_PAYME_8B66531C",
        "Use one record per original distribution and combine its 2026 payments; do not duplicate the repayment ceiling.",
    ),
    (
        "ISSUE_USE_ONE_RESOLVED_SE_SOURCE_PER_PARTNERSHIP_137CA69B",
        "Use one resolved SE source per partnership.",
    ),
    (
        "ISSUE_USE_THE_PAYER_S_TAXABLE_AMOUNT_AFTER_THE_SPECIAL_PRETAX_FIRST_ALLOC_E414AE3F",
        "Use the payer's taxable amount after the special pretax-first allocation, or a fully taxable or completely discharged contract. The ordinary proportional partial-withdrawal worksheet does not perform this allocation.",
    ),
    (
        "MESSAGE_MISSED_REQUIRED_DISTRIBUTION_ENTER_A_NAME_THE_AMOUNTS_CORRECTION_DATES_IN_20",
        "Missed required distribution: enter a name, the amounts, correction dates in 2026 or later and a waiver of at least $0.01.",
    ),
    (
        "MESSAGE_MISSED_REQUIRED_DISTRIBUTION_YOUR_SPOUSE_S_GO_ON_THIS_RETURN_ONLY_WHEN_YOU_F",
        "Missed required distribution: your spouse's go on this return only when you file jointly.",
    ),
    (
        "MESSAGE_ROTH_IRA_HISTORY_YOUR_SPOUSE_S_GOES_ON_THIS_RETURN_ONLY_WHEN_YOU_FILE_JOINTL",
        "Roth IRA history: your spouse's goes on this return only when you file jointly.",
    ),
    (
        "MESSAGE_SOCIAL_SECURITY_FOR_EARLIER_YEARS_ENTER_EACH_EARLIER_YEAR_1984_2025_ONCE_WIT",
        "Social Security for earlier years: enter each earlier year (1984–2025) once, with at most 42 years.",
    ),
    (
        "IRA_REPAYMENT_AMOUNT_UNREPRESENTABLE",
        "An IRA basis or repayment amount cannot be represented in the native reported result. Review the original distribution and repayment sources.",
    ),
    (
        "MESSAGE_YOUR_SPOUSE_S_CONTRIBUTIONS_BELONG_ON_THIS_RETURN_ONLY_WHEN_YOU_FILE_JOINTLY",
        "Your spouse's contributions belong on this return only when you file jointly.",
    ),
    (
        "PROBLEM_UNDISPLAYABLE_IRA_RECORD",
        "This return holds an IRA record this page cannot show yet. Use a fresh development task.",
    ),
    (
        "R_IRA_DISABILITY_NOT_WAGES",
        "An IRA distribution is never wages; choose the first answer for the minimum retirement age.",
    ),
    (
        "R_DISABILITY_WAGES_ABOVE_TAXABLE",
        "The payments before the minimum retirement age cannot exceed the taxable amount in box 2a.",
    ),
    ("R_TAXABLE_ABOVE_GROSS", "Box 2a cannot exceed box 1 gross distribution."),
    ("R_WITHHOLDING_ABOVE_GROSS", "Box 4 cannot exceed box 1 gross distribution."),
    ("R_BOX7_NO_CODE", "Enter the code printed in box 7a."),
    (
        "R_BOX7_TOO_MANY_CODES",
        "Box 7a shows at most two codes; enter only the codes printed there.",
    ),
    (
        "R_BOX7_CODES_NOT_COMBINED",
        "These box 7a codes are never printed together on one 1099-R, and code Y is always printed with 4, 7 or K. Check the codes printed on the form.",
    ),
    (
        "R_BOX7B_NOT_IRA",
        "Box 7b IRA/SEP/SIMPLE is never checked with code 6, 9, B, C, D, F, H, L, M, N, R, U or W: an IRA has no such contract, loan, designated Roth account or recharacterization code. Check the form.",
    ),
    (
        "R_BOX7B_NEEDED",
        "Codes K, S and Y are for a traditional or SIMPLE IRA, so box 7b IRA/SEP/SIMPLE is checked with them. Check the form.",
    ),
    (
        "R_BOX7_TREATMENT",
        "Choose what the box 7a codes mean: 7 normal, 1 or 2 early, 3 disability, 4 death (box 7b unchecked), 4 with box 7b an inherited IRA, 4 or G with box 7c a Trump account, G or H direct rollover, S SIMPLE IRA, Q, J or T Roth IRA, 6 or W exchange or long-term care charges, 8 corrective distribution, N, P or R not 2026 income, 9, E, F or U taxable as printed, 5 prohibited transaction (box 7b checked), A the 10-year tax option; B, D, K, L, M and Y follow the code printed with them, and D alone is a nonqualified annuity. For any other code, Another code.",
    ),
    (
        "R_BOX2B_METHOD",
        "Choose a method that fits the box 7a codes and box 7b; box 2a as printed always fits.",
    ),
    (
        "R_BOX7_EXCHANGE_TAXABLE",
        "A section 1035 exchange (code 6), long-term care charges (W), a recharacterization (N) or an earlier year's amount (P, R) show nothing taxable in box 2a. A taxable part comes on its own Form 1099-R with another code.",
    ),
    (
        "R_CARE_CHARGE_SOURCE_INVALID",
        "For a qualified long-term-care cash-value charge use the actual code W (alone or with 6), box 2a zero, no IRA/Trump checkbox or other taxable treatment, and the reported box 8a investment reduction no greater than gross. Keep other retirement boxes and premium deductions separate.",
    ),
    (
        "R_EXCHANGE_OTHER_BOXES",
        "For a section 1035 exchange leave a qualified disaster distribution, disability pay and public safety premiums empty.",
    ),
    (
        "R_NONQUALIFIED_KEPT",
        "A nonqualified annuity (code D) isn't rolled over or converted. Choose Kept it.",
    ),
    (
        "PROPERTY_ROLLOVER_NOT_FROM_A_PLAN",
        "Property sold for a rollover must come from an employer plan, not from an IRA or a nonqualified annuity.",
    ),
    (
        "PROPERTY_ROLLOVER_AFTER_TAX",
        "Property sold for a rollover needs box 2a equal to box 1 and box 2b answered 'Box 2a shows the taxable amount'.",
    ),
    (
        "PROPERTY_ROLLOVER_ABOVE_PROCEEDS",
        "The amount you deposited is more than the sale proceeds.",
    ),
    (
        "PROPERTY_SOLD_BEFORE_DISTRIBUTION",
        "The property was sold before it was distributed to you.",
    ),
    (
        "R_WITHDRAWAL_ABOVE_BALANCE",
        "The payment in box 1 is more than the account or contract value you entered.",
    ),
    (
        "CSRS_RECOVERED_ABOVE_COST",
        "The earlier tax-free recovery is more than the employee's cost.",
    ),
    ("R_TRUMP_AND_IRA", "A Trump account is not an IRA for this form: leave box 7b unchecked."),
    ("R_TRUMP_BRANCH_NEEDS_BOX_7C", "A Trump account choice needs box 7c checked."),
    (
        "R_TRUMP_ABLE_TAXABLE",
        "A transfer of the whole Trump account to an ABLE account is not taxed: box 2a is zero.",
    ),
    (
        "R_TRUMP_CODES",
        "A Trump account (box 7c) is printed with code 4, G or 8 alone. Check the form.",
    ),
    (
        "R_BOX7_H_TAXABLE",
        "A direct rollover of a designated Roth account to a Roth IRA (code H) shows nothing taxable in box 2a. Check the form, or use code G.",
    ),
    (
        "R_BOX7_H_TO_ROTH",
        "Code H is a direct rollover to a Roth IRA or designated Roth account. Choose that destination.",
    ),
    (
        "R_DRAC_DIRECT_ROLLOVER",
        "A designated Roth account distribution (code B) that is rolled over directly goes to another designated Roth account (an employer plan) or a Roth IRA. Choose one of them.",
    ),
    (
        "R_ROTH_ROLLOVER_FROM_A_PLAN",
        "Only money from an employer plan is rolled into a Roth account here. Money from an IRA goes to a Roth IRA as a conversion, and a SIMPLE IRA cannot be rolled to a Roth IRA in its first two years.",
    ),
    (
        "R_DRAC_ROLLOVER_TO_ROTH",
        "A designated Roth account distribution (code B) that you rolled over within 60 days can go only to a Roth IRA. Choose that destination, or Kept it.",
    ),
    (
        "R_LOAN_KEPT",
        "A loan treated as a distribution (code L) cannot be rolled over. Choose Kept it.",
    ),
    (
        "R_EXCESS_DEFERRAL_ABOVE_TAXABLE",
        "The part of box 2a that is an excess deferral cannot exceed box 2a.",
    ),
    (
        "R_EXCESS_DEFERRAL_NOT_A_PLAN",
        "An excess deferral comes from a 401(k), 403(b) or salary-reduction SEP plan, not an IRA or a designated Roth account. Leave this part at 0.",
    ),
    (
        "R_SPECIAL_CODE_OTHER_BOXES",
        "For codes 8, E, F, N, P, R, 9 and U leave the box 2b method, the qualified disaster distribution and public safety premiums empty.",
    ),
    (
        "R_BOX10_WITHOUT_B",
        "Box 10 is for a designated Roth account distribution (code B). Leave it empty with other codes.",
    ),
    ("R_BOX10_ABOVE_GROSS", "Box 10 cannot exceed box 1 gross distribution."),
    (
        "R_BOX10_BASIS_WITHOUT_BOX10",
        "The part of box 10 not taxable when rolled in belongs only to a code B distribution with a box 10 amount.",
    ),
    (
        "R_BOX10_BASIS_ABOVE_BOX10",
        "The part of box 10 not taxable when rolled in cannot exceed box 10.",
    ),
    (
        "R_REPAYMENT_NOT_SPECIAL",
        "A repayment is calculated only for a birth or adoption, emergency, domestic-abuse or terminal-illness distribution from an IRA. A repayment of an employer-plan distribution is not calculated yet, so that distribution stays taxable: leave the repayment empty.",
    ),
    ("R_REPAYMENT_ABOVE_GROSS", "The deposits repaid cannot exceed box 1 gross distribution."),
    (
        "SEPP_FIRST_PAYMENT_AFTER_DISTRIBUTION",
        "The first payment of the series cannot be after this distribution.",
    ),
    (
        "LUMP_SUM_DEATH_BENEFIT_AFTER_1996",
        "The death benefit exclusion is only for a participant who died before August 21, 1996. Leave your share at 0.",
    ),
    (
        "R_DISASTER_NONQUALIFIED",
        "A nonqualified annuity (code D) is not an eligible retirement plan for a qualified disaster distribution. Choose No.",
    ),
    (
        "DISASTER_OUTSIDE_PERIOD",
        "The distribution is outside the qualified disaster distribution period, which runs from the incident beginning date through 179 days after the later of the beginning and declaration dates. It is not a qualified disaster distribution: choose No.",
    ),
    (
        "DISASTER_DIRECT_ROLLOVER",
        "A direct rollover is not a qualified disaster distribution. Choose No.",
    ),
    (
        "DISASTER_AND_ROLLOVER_ABOVE_GROSS",
        "The amount you treat as a qualified disaster distribution and the amount you rolled over or converted cannot add up to more than box 1 gross distribution.",
    ),
    (
        "DISASTER_HOME_OUTSIDE_PERIOD",
        "A distribution to buy or build a main home must be received no more than 180 days before the first day and no later than 30 days after the last day of the disaster.",
    ),
    (
        "DISASTER_ENDED_BEFORE_BEGAN",
        "The disaster ending date cannot be before its beginning date.",
    ),
    (
        "DISASTER_HOME_REPAID_ABOVE_TAXABLE",
        "The taxable part repaid cannot exceed box 2a taxable amount.",
    ),
    (
        "DISASTER_HOME_KEPT",
        "A distribution for a main home is taxed with your other income. Choose the normal or early distribution meaning for the box 7a codes.",
    ),
    (
        "IRA_ROLLOVER_ONCE_A_YEAR",
        "Only one rollover from an IRA to an IRA is tax-free in any 12 months across all of a person's IRAs. Convert the other to a Roth IRA, move it by a direct transfer between trustees, or keep it.",
    ),
    (
        "R_EXCESS_DEFERRAL_ABOVE_W2_EXCESS",
        "The excess deferrals returned are more than the excess the App counts from this person's W-2 box 12 deferrals. Check the plan's letter and the W-2s on Income.",
    ),
    (
        "QCD_SPLIT_INTEREST_ABOVE_LIMIT",
        "The one-time gifts to a split-interest entity cannot add up to more than $55,000 for one person.",
    ),
    (
        "DISASTER_ABOVE_QCD_REST",
        "A gift to charity is excluded first. The qualified disaster distribution and the gifts to charity cannot add up to more than the IRA distributions.",
    ),
    (
        "ISSUE_SEPP_RECAPTURE",
        "A change in a series of substantially equal periodic payments before the later of five years after the first payment and age 59½ adds the earlier years' 10% additional tax with interest, which is not calculated.",
    ),
    (
        "R_INHERITED_OTHER_BOXES",
        "For an inherited IRA or a Trump account leave a qualified disaster distribution, disability pay and public safety premiums empty.",
    ),
    ("RRB_BOX7_NOT_SUM", "Box 7 should equal boxes 4, 5 and 6 added together. Check the form."),
    ("RRB_REPAYMENTS_WITHOUT_BOX_8", "Leave the repayments empty when box 8 is zero."),
    ("RRB_BOX_8_NOT_SPLIT", "Enter box 8 by the year the benefits were paid."),
    ("RRB_REPAYMENTS_NOT_BOX_8", "The repayments by year should add up to box 8."),
    ("R_ROTH_CODE_NOT_BOX7", "Choose the Roth IRA code printed in box 7a: Q, J or T."),
    ("R_BOX3_ABOVE_TAXABLE", "Box 3 capital gain cannot exceed box 2a taxable amount."),
    ("R_BOX8B_WITHOUT_BOX8A", "Box 8b needs the box 8a annuity contract value."),
    (
        "LUMP_SUM_CAPITAL_GAIN_WITHOUT_BOX_3",
        "The 20% capital gain rate needs a box 3 capital gain above zero. Choose the 10-year tax option alone, or enter box 3.",
    ),
    (
        "LUMP_SUM_NUA_WITHOUT_BOX_6",
        "Including the net unrealized appreciation needs a box 6 amount above zero.",
    ),
    (
        "LUMP_SUM_OTHER_TREATMENTS",
        "A lump sum taxed on Form 4972 cannot also use the Simplified Method, box 10, a disability pension, a qualified disaster distribution or public safety officer premiums.",
    ),
    ("LUMP_SUM_FIRST_PAYMENT_NOT_IN_2026", "The first payment of the lump sum must fall in 2026."),
    (
        "LUMP_SUM_PRIOR_PAYMENT_NOT_EARLIER",
        "The latest earlier payment must be before 2026. Payments in 2026 belong to this lump sum.",
    ),
    (
        "LUMP_SUM_BORN_AFTER_DEATH",
        "The participant's date of birth must be before the date of death.",
    ),
    ("ROTH_HISTORY_TWICE", "Enter the Roth IRA history only once for each owner."),
    (
        "ROTH_CONVERSION_YEAR_TWICE",
        "Enter each conversion year once; add together what you converted that year.",
    ),
    (
        "ROTH_CONVERSION_BEFORE_FIRST_YEAR",
        "A conversion year cannot be before your first Roth IRA year.",
    ),
    (
        "ROTH_DISABLED_AFTER_DISTRIBUTION",
        "The date you became disabled cannot be after the distribution.",
    ),
    (
        "R_ROTH_OTHER_BOXES",
        "For a Roth IRA (code J, Q or T) leave the other boxes empty: the Simplified Method, box 10 and disability pay do not apply. Box 9a needs nothing here.",
    ),
    (
        "R_DISASTER_ABOVE_GROSS",
        "The qualified disaster distribution cannot exceed box 1 gross distribution.",
    ),
    (
        "DISASTER_REPAID_ABOVE_AMOUNT",
        "You cannot repay more than the qualified disaster distribution.",
    ),
    (
        "DISASTER_REPAID_2026_ABOVE_REPAID",
        "The part repaid during 2026 cannot exceed the total repaid.",
    ),
    (
        "FORM_8915_DISASTER_AFTER_FORM",
        "A 2024 Form 8915-F reports only disasters that began in 2024 or earlier (item B).",
    ),
    (
        "FORM_8915_CARRIED_BACK_ABOVE_EXCESS",
        "The amount carried back cannot exceed the 2025 form's excess repayment (line 14 over line 13, or line 25 over line 24).",
    ),
    (
        "PSO_NOT_FROM_AN_IRA",
        "The retired public safety officer exclusion applies to a pension from a governmental plan, not an IRA or Roth IRA (box 7b IRA/SEP/SIMPLE, codes J, Q and T).",
    ),
    (
        "PSO_FUNDED_ABOVE_PREMIUMS",
        "The part paid from this pension cannot exceed the premiums paid.",
    ),
    (
        "PSO_FUNDED_ABOVE_GROSS",
        "The premiums paid from this pension cannot exceed its gross distribution (box 1).",
    ),
    (
        "LUMP_SUM_ELECTION_TWICE",
        "Enter the Social Security paid for earlier years once for the whole return.",
    ),
    ("LUMP_SUM_YEAR_TWICE", "Enter each earlier year once."),
    (
        "LUMP_SUM_TAXABLE_ABOVE_BENEFITS",
        "The taxable benefits on the earlier return cannot exceed that year's benefits.",
    ),
    (
        "LUMP_SUM_REFIGURED_ABOVE_BENEFITS",
        "The refigured taxable benefits cannot exceed that year's benefits and earlier lump sums.",
    ),
    (
        "NUA_SOLD_BEFORE_DISTRIBUTED",
        "NUA shares sold: the date sold (1099-B box 1c) cannot be before the date the plan distributed the shares.",
    ),
    ("NUA_SALE_NOT_IN_2026", "NUA shares sold: enter a sale made in 2026 (1099-B box 1c)."),
    (
        "NUA_ABOVE_DISTRIBUTION_VALUE",
        "NUA shares sold: the net unrealized appreciation cannot be more than the value of the shares when they were distributed.",
    ),
    (
        "NUA_DIED_BEFORE_DISTRIBUTED",
        "NUA shares sold: the previous owner cannot have died before the plan distributed the shares.",
    ),
    (
        "NUA_SOLD_BEFORE_DIED",
        "NUA shares sold: the date sold (1099-B box 1c) cannot be before the previous owner's date of death.",
    ),
    (
        "IRA_NONDEDUCTIBLE_ABOVE_TRADITIONAL",
        "The nondeductible election cannot exceed the traditional IRA contribution.",
    ),
    (
        "IRA_CONTRIBUTOR_TWICE",
        "Enter one entry for each owner; combine all of an owner's IRA contributions into that entry.",
    ),
    (
        "IRA_EXCESS_TRADITIONAL_VALUE_NEEDED",
        "Enter the December 31, 2026 value of your traditional IRAs.",
    ),
    (
        "IRA_EXCESS_TRADITIONAL_ON_8606",
        "Leave the traditional IRA value empty: your Form 8606 on the Retirement page already gives it.",
    ),
    (
        "ISSUE_INHERITED_DISASTER_OVERLAP",
        "The group's qualified disaster distributions cannot include amounts already excluded for a qualified charitable distribution or qualified HSA funding. Reconcile the actual linked payments.",
    ),
    (
        "NOTE_FORM_8606_DIVORCE_STATEMENT",
        "If any of the Form 8606 line 2 adjustment is basis received or transferred in a divorce, attach a statement to Form 8606 explaining it, with the other spouse's name.",
    ),
    (
        "NOTE_ROTH_REPAYMENT_NEXT_YEAR_BASIS",
        "{0}: {1,number,currency} of the Roth repayment exceeds this year's taxable amount; add it to line 22 of your next Form 8606 Part III (line 5 of the 2026 Line 25c Worksheet).",
    ),
    (
        "NOTE_LINE_15C_NEXT_YEAR_BASIS",
        "{0}: {1,number,currency} of the repayment exceeds this year's taxable amount; add it to line 2 of your next Form 8606 (line 6 of the 2026 Line 15c Worksheet).",
    ),
    (
        "FORM_8606_ADJUSTMENT_BELOW_BASIS",
        "The adjustment cannot take the Form 8606 basis below zero: a decrease is limited to line 14 plus any increase.",
    ),
    (
        "RESERVIST_DISTRIBUTION_TWICE",
        "Enter each original reservist distribution once, with all its 2026 repayments.",
    ),
    (
        "RESERVIST_DISTRIBUTION_OUTSIDE_DUTY",
        "The distribution date must fall from the day you were ordered to active duty through the last day of the active-duty period.",
    ),
    (
        "RESERVIST_REPAYMENT_LATE",
        "A reservist repayment must follow the end of the active-duty period and fall within two years after it.",
    ),
    (
        "RESERVIST_REPAYMENT_ABOVE_GROSS",
        "Repayments of a reservist distribution cannot add to more than the distribution.",
    ),
    (
        "IRA_RETURN_AFTER_DUE_DATE_NEEDS_ROUTE",
        "A contribution returned after April 15, 2027 needs a filing extension or a timely filed return to be a returned contribution. Choose which applied.",
    ),
    (
        "IRA_RETURN_ROUTE_WITHOUT_LATE_DATE",
        "A return by April 15, 2027 needs no extension or filing route. Leave that choice empty.",
    ),
    (
        "IRA_RETURN_LOSS_ABOVE_PRINCIPAL",
        "The earnings or loss cannot take the cash returned below zero.",
    ),
    (
        "IRA_RETURNED_EXCESS_ABOVE_CARRIED",
        "The excess paid back in 2026 cannot be more than the traditional IRA excess carried from your 2025 Form 5329.",
    ),
    (
        "IRA_RETURNED_EXCESS_ABOVE_DISTRIBUTIONS",
        "The excess paid back in 2026 cannot be more than your traditional IRA withdrawals in 2026 on the Retirement page.",
    ),
    (
        "ROTH_ROLLOVER_ONCE_A_YEAR",
        "Only one rollover from an IRA to an IRA is tax-free in any 12 months across all of a person's IRAs, Roth IRAs included. Move the other by a direct transfer between trustees, or keep it.",
    ),
    (
        "R_ROTH_REPAYMENT_NEEDS_HISTORY",
        "A repayment of a Roth IRA distribution is figured on your Roth IRA history. Enter it on this page first.",
    ),
    (
        "R_ROTH_SEQUENCE_INVALID",
        "Enter an actual same-day payment order from 1 through 100, or leave it absent when no HSA funding shares the date.",
    ),
    (
        "R_ROTH_ROLLOVER_ABOVE_GROSS",
        "The amount rolled over to another Roth IRA cannot be more than box 1.",
    ),
    ("R_ROLLOVER_ABOVE_GROSS", "The amount rolled over cannot be more than box 1."),
    ("R_CONVERSION_ABOVE_GROSS", "The amount converted cannot be more than box 1."),
    (
        "EMERGENCY_REPAID_ABOVE_DISTRIBUTION",
        "The earlier emergency withdrawal's repayment cannot be more than that withdrawal.",
    ),
    (
        "R_ROTH_EXCEPTION_NOT_IRA",
        "That exception applies only to a distribution from an employer plan. A Roth IRA can use disability, a first home, education, medical expenses, health insurance while unemployed, a birth or adoption, an emergency, terminal illness, domestic abuse, reservist duty, periodic payments or an IRS levy.",
    ),
    (
        "R_ROTH_DISASTER_KIND",
        "A Roth IRA distribution can be a qualified disaster distribution; a main home purchase in a disaster area (Form 8915-F Part IV) does not apply to it.",
    ),
    (
        "R_EMPLOYER_ROTH_CONTRIBUTION",
        "An employer's Roth SEP or Roth SIMPLE contribution has box 7b checked and box 2a equal to box 1. It has no other special treatment.",
    ),
    (
        "R_SECOND_EXCEPTION_KIND",
        "Periodic payments and age 59½ each cover the whole distribution; they are not a second exception.",
    ),
    (
        "R_COST_NEEDS_BOX_9B",
        "Enter the original after-tax cost, or leave it empty only when box 9b is entered and the annuity started in 2026.",
    ),
];
