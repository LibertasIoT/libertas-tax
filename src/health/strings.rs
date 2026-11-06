pub(crate) const HEALTH_STRINGS: &[(&str, &str)] = &[
    ("DISPLAY_HEALTH_STATEMENT_YEAR", "Year and status"),
    ("DISPLAY_HEALTH_STATEMENT_TRUSTEE", "Trustee"),
    ("DISPLAY_HEALTH_STATEMENT_PARTICIPANT", "Participant"),
    ("DISPLAY_HEALTH_STATEMENT_BOXES", "Contributions and account value"),
    (
        "HEALTH_STATEMENT_ACCOUNT_REQUIRED",
        "Enter this statement owner's actual HSA or Archer MSA eligibility and contribution records.",
    ),
    (
        "HEALTH_STATEMENT_SPLIT_REQUIRED",
        "Use custodian and employer records to establish the contribution-year and employer portions of this Form 5498-SA.",
    ),
    (
        "HEALTH_STATEMENT_SPLIT_INVALID",
        "Reconcile these contribution classifications with the printed statement amounts.",
    ),
    (
        "HEALTH_STATEMENT_TOTAL_INVALID",
        "The actual health-account source totals cannot be represented. Review the source amounts.",
    ),
    (
        "HEALTH_STATEMENT_W2_INVALID",
        "Choose the actual same-owner W-2 and only its current-year contribution portion included in this statement.",
    ),
    (
        "HEALTH_STATEMENT_FUNDING_INVALID",
        "Choose each actual qualified IRA-to-HSA transfer included in this statement once, using its original payment and funding date.",
    ),
    (
        "HEALTH_STATEMENT_DEPOSIT_INVALID",
        "Choose each actual same-owner failed-rollover deposit included in this statement once.",
    ),
    (
        "HEALTH_STATEMENT_SOURCES_CHANGED",
        "The health-account statements changed. Reopen this page to review the current sources.",
    ),
    (
        "LTC1099_CONTRACT_CHECK_CONFLICT",
        "The retained contract answer conflicts with the printed box 4 check. Correct the actual contract answer.",
    ),
    (
        "HEALTH_EMPLOYER_YEAR_REQUIRED",
        "Choose the actual contribution year for each entered W-2 medical-account employer amount before completing this estimate.",
    ),
    (
        "HEALTH_EMPLOYER_ACCOUNT_REQUIRED",
        "Enter the matching owner’s current HSA or Archer MSA account facts for a current-year employer contribution.",
    ),
    (
        "HEALTH_EMPLOYER_SPLIT_ABOVE_REPORTED",
        "The current-year part cannot exceed the actual W-2 code amount; the remainder belongs to the prior year.",
    ),
    (
        "HEALTH_EMPLOYER_TOTAL_INVALID",
        "Employer contribution amounts must be nonnegative and fit the supported total.",
    ),
    (
        "HEALTH_EMPLOYER_SOURCE_NOT_OFFERED",
        "Use every actual entered W-2 medical-account contribution source exactly once.",
    ),
    (
        "HEALTH_EMPLOYER_SOURCE_TWICE",
        "This W-2 medical-account contribution source can be allocated only once.",
    ),
    (
        "HEALTH_EMPLOYER_SOURCE_CHANGED",
        "Review the W-2 contribution-year allocation before changing its source amount. Clear an accepted split on Health and medical savings accounts before changing the linked W-2.",
    ),
    (
        "HEALTH_EMPLOYER_SOURCE_REFERENCED",
        "Clear this W-2’s contribution-year allocation on Health and medical savings accounts before removing or changing its owner, source kind or code.",
    ),
    (
        "HEALTH_SOURCE_AMOUNTS_UNAVAILABLE",
        "The combined health premium source amounts exceed the supported arithmetic range.",
    ),
    ("DISPLAY_HSA_ACCOUNT", "The account"),
    ("DISPLAY_HSA_FAMILY", "Eligibility and family allowance"),
    ("DISPLAY_ARCHER_CONTRIBUTIONS", "Archer MSA contributions"),
    ("DISPLAY_LTC_AMOUNTS", "Amounts on this receipt"),
    ("DISPLAY_LTC_CHARACTER", "Payment and contract treatment"),
    ("DISPLAY_LTC_INSURED", "Insured and care periods"),
    ("DISPLAY_HSA_EXCESS", "Excess contributions"),
    ("DISPLAY_HSA_OTHER", "Rollover, prohibited transactions and earlier years"),
    ("HEALTH_RETURN_DOCUMENT_CHOICE", "{0} — gross {1,number,currency}"),
    (
        "HEALTH_CORRECTION_DOCUMENT_UNAVAILABLE",
        "Select this owner's actual accepted returned-excess statement, or leave the link empty for a manual correction.",
    ),
    (
        "HEALTH_CORRECTION_DOCUMENT_DETAILS_REQUIRED",
        "Complete the contribution correction for each entered HSA returned-excess statement or Archer statement without printed earnings, including its principal, signed attributable income, contribution year and payer.",
    ),
    (
        "HEALTH_CORRECTION_DOCUMENT_YEAR",
        "The linked statement is for this tax year; its correction must have been returned during that year.",
    ),
    (
        "HEALTH_CORRECTION_DOCUMENT_GROSS",
        "The principal and signed attributable income of the linked correction rows must equal the statement's actual gross cash returned.",
    ),
    (
        "HEALTH_CORRECTION_DOCUMENT_EARNINGS",
        "The linked corrections' positive actual earnings must reconcile to the statement's printed earnings.",
    ),
    (
        "HSA_CORRECTION_RETURNED_BEFORE_DEPOSIT",
        "The excess cannot be returned before it was deposited.",
    ),
    (
        "HSA_CORRECTION_DEADLINE_BEFORE_DEPOSIT",
        "The correction deadline cannot be before the deposit.",
    ),
    ("HSA_CORRECTION_LOSS_ABOVE_PRINCIPAL", "A loss cannot be more than the principal returned."),
    (
        "HSA_LATE_EXCESS_NOT_LATE",
        "The distribution must come after the correction deadline; an earlier one is a timely correction.",
    ),
    (
        "HSA_PRIOR_EXCESS_DEPOSIT_AFTER_DEADLINE",
        "The deposit cannot be after the last permitted date for a 2025 contribution.",
    ),
    (
        "HSA_LATER_EMPLOYER_DEPOSITS_ABOVE_LATER",
        "The employer part of the later deposits cannot be more than all the later deposits.",
    ),
    (
        "HSA_FUNDING_LOSS_BEFORE_FUNDING",
        "The first loss of eligibility cannot be before the transfer.",
    ),
    ("HSA_FUNDING_NEEDS_COVERAGE", "A transfer needs eligible coverage when it was made."),
    (
        "HSA_FUNDING_SOURCE_NOT_OFFERED",
        "Choose an existing same-owner IRA Form 1099-R payment. An inherited payment must retain its valid beneficiary/deceased-owner group; the annual group itself is not a payment.",
    ),
    (
        "HSA_FUNDING_ONGOING_SEP_SIMPLE",
        "An ongoing SEP or SIMPLE IRA cannot fund a qualified HSA transfer. An employer contribution for the plan year ending in the distribution year makes that funding account ongoing, even if the contribution was made later. Correct the source treatment; keep the actual IRA distribution reported.",
    ),
    (
        "HSA_FUNDING_DISTRIBUTION_YEAR",
        "Enter the actual valid 2026 IRA distribution date. The HSA receipt date and a contribution designation for another year do not change the distribution year.",
    ),
    (
        "HSA_FUNDING_DISTRIBUTION_AFTER_RECEIPT",
        "The IRA distribution cannot be after the HSA received that transfer. Correct the actual source date or HSA receipt date.",
    ),
    (
        "HSA_FUNDING_BEFORE_INHERITED_DEATH",
        "An inherited IRA funding distribution cannot be before the original owner's death. Correct the actual payment date or its inherited source link.",
    ),
    (
        "HSA_FUNDING_OWNER_DIED_BEFORE_RECEIPT",
        "This HSA receipt was after the account owner's death. This transfer needs source treatment the App cannot establish here. Keep its actual IRA distribution date and income source; the receipt does not change that date.",
    ),
    (
        "HSA_FUNDING_LIFETIME",
        "Qualified IRA-to-HSA funding is once per lifetime. A second distribution is allowed only in a later month of the same year after self-only coverage changes to family coverage; use the actual IRA distribution dates, not HSA receipt dates.",
    ),
    (
        "HSA_FUNDING_ORDER_CONTRADICTED",
        "The funding-first answer disagrees with an earlier ordinary Roth withdrawal in the same group. Correct the answer in Health accounts or the actual withdrawal date in Retirement.",
    ),
    (
        "HSA_FUNDING_NEEDS_AN_ACCOUNT",
        "Add the owner's health savings account before the transfers from an IRA to it.",
    ),
    (
        "HSA_RECORD_INVALID",
        "Check the dates and amounts of this health savings account: corrections, deposits, transfers and the rollover must be valid and fit within the year.",
    ),
    (
        "HSA_CORRECTED_PERSONAL_ABOVE_CONTRIBUTED",
        "The personal excess returned for 2026 cannot be more than the personal contributions.",
    ),
    (
        "HSA_CORRECTED_EMPLOYER_ABOVE_CONTRIBUTED",
        "The employer excess returned for 2026 cannot be more than the employer contributions.",
    ),
    (
        "HSA_LATER_EMPLOYER_ABOVE_NET",
        "The employer part of the later deposits cannot be more than the employer contributions left after the excess returned.",
    ),
    (
        "HSA_LATER_PERSONAL_ABOVE_NET",
        "The personal part of the later deposits cannot be more than the personal contributions left after the excess returned.",
    ),
    ("HSA_FUNDING_HISTORY_SAME_MONTH", "Two IRA transfers cannot both be in the same month."),
    (
        "HSA_LAST_MONTH_DECEMBER_NEEDED",
        "The last-month rule needs eligible coverage in December 2025.",
    ),
    (
        "ARCHER_CORRECTION_RETURNED_BEFORE_DEPOSIT",
        "The excess cannot be returned before it was deposited.",
    ),
    (
        "ARCHER_CORRECTION_DEADLINE_BEFORE_DEPOSIT",
        "The correction deadline cannot be before the deposit.",
    ),
    (
        "ARCHER_CORRECTION_LOSS_ABOVE_PRINCIPAL",
        "A loss cannot be more than the principal returned.",
    ),
    (
        "ARCHER_PRIOR_EXCESS_DEPOSIT_AFTER_DEADLINE",
        "The deposit cannot be after the last permitted date for a 2025 contribution.",
    ),
    (
        "ARCHER_LATER_EMPLOYER_DEPOSITS_ABOVE_LATER",
        "The employer part of the later deposits cannot be more than all the later deposits.",
    ),
    ("ARCHER_OWNER_TWICE", "Enter each person's Archer MSA contributions once."),
    (
        "ARCHER_RECORD_INVALID",
        "Check the Archer MSA contributions: the dates and amounts must be valid, and each owner's coverage needs twelve months.",
    ),
    (
        "ARCHER_CORRECTED_PERSONAL_ABOVE_CONTRIBUTED",
        "The personal excess returned for 2026 cannot be more than the personal contributions.",
    ),
    (
        "ARCHER_CORRECTED_EMPLOYER_ABOVE_CONTRIBUTED",
        "The employer excess returned for 2026 cannot be more than the employer contributions.",
    ),
    (
        "ARCHER_LATER_EMPLOYER_ABOVE_NET",
        "The employer part of the later deposits cannot be more than the employer contributions left after the excess returned.",
    ),
    (
        "ARCHER_LATER_PERSONAL_ABOVE_NET",
        "The personal part of the later deposits cannot be more than the personal contributions left after the excess returned.",
    ),
    (
        "ARCHER_INCOME_NOT_OFFERED",
        "Choose compensation that is still on your return: a W-2, a Form 8919 or a business.",
    ),
    ("ARCHER_INCOME_TWICE", "Choose each compensation source once."),
    ("SOURCE_DOCUMENT_CHOICE", "{0} ({1})"),
    (
        "SA1099_MEDICAL_ABOVE_GROSS",
        "The money spent on medical costs and the rollover cannot be more than the distribution (box 1).",
    ),
    (
        "SA1099_ROLLOVER_NOT_ARCHER",
        "Only an HSA or Archer MSA payment can be rolled over to an eligible account of the same owner.",
    ),
    (
        "SA1099_ROLLOVER_BEFORE_RECEIPT",
        "The rollover cannot be deposited before the payment was received.",
    ),
    (
        "SA1099_UNREPORTED_SOURCE_INVALID",
        "Use a trustee statement for an unreported HSA or Archer MSA account event; this is not a printed Form 1099-SA code.",
    ),
    (
        "SA1099_EVENT_VALUE_NOT_GROSS",
        "The statement's gross distribution must equal the terminated account's January1 value or the total fair market value of the pledged portions.",
    ),
    (
        "SA1099_USES_NOT_GROSS",
        "Medical spending, the rollover and all dated nonmedical payments must equal this statement's gross distribution (box 1).",
    ),
    (
        "SA1099_EXCESS_NOT_ARCHER",
        "Only an HSA or Archer MSA statement can report a returned excess contribution.",
    ),
    (
        "SA1099_EARNINGS_ABOVE_GROSS",
        "The earnings on the excess (box 2) cannot be more than the distribution (box 1).",
    ),
    (
        "SA1099_SPOUSE_ACCOUNT_IS_OWN",
        "A Medicare Advantage MSA inherited from your spouse is your own account: enter its payments as paid from your own account.",
    ),
    ("LTC1099_NO_AMOUNT", "Enter the benefits (box 1) or the accelerated death benefits (box 2)."),
    (
        "LTC1099_DATE_CERTIFIED_INVALID",
        "Enter the actual calendar date printed in box 5, or select that the date is not reported or unavailable.",
    ),
    ("LTC_PERIOD_ENDS_BEFORE_START", "The last day of care cannot be before the first day."),
    (
        "SA1099_PRINTED_CODE_NEEDED",
        "Enter the printed box 3 code for a Form 1099-SA, or leave it empty for a trustee-only event.",
    ),
    (
        "SA1099_DEATH_VALUE_NEEDED",
        "Enter the printed box 4 value for this inherited-account distribution.",
    ),
    (
        "SA1099_PAYMENT_NOT_PRINTED",
        "Reconcile the retained payment records with the printed distribution code.",
    ),
    (
        "ISSUE_MEDICARE_MSA_PRIOR_DIFFERS",
        "Two Medicare Advantage MSA forms for the same person show different December 31, 2025 balances. Use the same account value on each.",
    ),
    (
        "LTC_FUNDING_HISTORY_INVALID",
        "Use all of the latest available group-policy years, up to three unique years, or the first-year method when none is known; do not use both.",
    ),
    (
        "LTC_FUNDING_ZERO",
        "The actual employee and employer net premiums must add up to more than zero.",
    ),
    (
        "LTC1099_ORIGIN_STREAM",
        "Use LTC insurance for box 1 and the actual life-policy advance or viatical settlement for box 2.",
    ),
    (
        "LTC1099_EXCLUDED_PORTIONS_ABOVE_EMPLOYER",
        "Medical reimbursement and permanent-injury portions cannot exceed the benefit attributable to tax-free employer payments.",
    ),
    (
        "LTC1099_SOURCE_UNAVAILABLE",
        "The linked annuity or prior-year recovery must be an actual same-owner source covering this receipt. Remove or change its LTC link before deleting or reassigning that source.",
    ),
    (
        "ISSUE_LTC_SOURCE_CHARACTER_UNRESOLVED",
        "Establish the actual source of the LTC insurance receipt before estimating its required taxable income.",
    ),
    (
        "ISSUE_LTC_LIFE_CHARACTER_UNRESOLVED",
        "The actual life-policy or viatical facts do not establish section 101(g) treatment. Its required income needs the existing life or annuity source calculation; the receipt cannot be treated as zero income.",
    ),
    (
        "ISSUE_LTC_NONQUALIFIED_CHARACTER_UNRESOLVED",
        "This nonqualified long-term-care receipt needs its actual accident/health, life or annuity tax treatment. Its mandatory taxable income is not established by Form 1099-LTC alone.",
    ),
    (
        "WARN_MARKETPLACE_SEPARATE_RELIEF_ASSUMED_NO",
        "Assumed the domestic-abuse or abandonment exception (Form 8962 line A) doesn't apply to your separate return, so no premium tax credit is allowed and the advance payments are repaid; if it applies, your tax may be lower.",
    ),
    (
        "HEALTH_PARTNERSHIP_POLICY_ON_CORPORATION",
        "A partnership-named policy paid by its partner is a partnership source. An S-corporation plan requires the corporation's actual payment or reimbursement and existing W-2 reporting.",
    ),
    (
        "HEALTH_REIMBURSED_CHILD_ABOVE_TOTAL",
        "The reimbursed child premium increase cannot exceed this month's total reimbursed ordinary premium.",
    ),
    (
        "HEALTH_CHILD_ABOVE_PREMIUMS",
        "The included child premium increase cannot exceed this month's total paid ordinary premium.",
    ),
    (
        "HEALTH_REIMBURSEMENT_ABOVE_SOURCE",
        "Actual reimbursement must fit this paid premium and its included child and medical portions. Review the source amounts; reimbursement is not another expense.",
    ),
    (
        "HEALTH_REIMBURSED_COVERAGE_ABOVE_TOTAL",
        "The reimbursed specified coverage-family premium cannot exceed the total reimbursed premium assigned to this business.",
    ),
    ("CHOOSE_WHOSE_DOCUMENT_THIS_IS", "Choose whose document this is"),
    (
        "MEDICAL_ROLLOVER_HISTORY_CONFLICT",
        "Use the same actual prior-year successful rollover receipt for this owner and source account type on every statement. Current-year successful history is derived from actual source entries.",
    ),
    (
        "MEDICAL_ROLLOVER_SAME_DAY_PRIORITY_UNRESOLVED",
        "Competing potentially tax-free rollover receipts on the same day need a verified source priority rule. The App does not choose one by statement identity or editor order; the affected complete estimate is unavailable.",
    ),
    (
        "MEDICAL_FAILED_PROPERTY_DEPOSIT_UNRESOLVED",
        "A failed property rollover is not an ordinary cash contribution. Its destination account-qualification treatment is unresolved; the affected complete estimate is unavailable.",
    ),
    (
        "MEDICAL_FAILED_PRIOR_YEAR_DEPOSIT_UNRESOLVED",
        "This failed deposit was designated for a prior year. Reconcile its actual prior-year deduction and any excess carried into2026; the App does not apply2026 contribution limits to that prior year.",
    ),
    (
        "MEDICAL_FAILED_DEPOSIT_ACCOUNT_REQUIRED",
        "Add this same owner and destination account type on Health accounts with actual eligibility, compensation and excess facts. The linked failed cash deposit is counted once as an ordinary contribution; keep it out of manual totals.",
    ),
    ("DISPLAY_HEALTH_CORRECTION_SOURCE", "Original contribution and source"),
    ("DISPLAY_HEALTH_CORRECTION_RETURN", "Completed corrective return"),
    (
        "HEALTH_FAILED_DEPOSIT_PERSONAL_REQUIRED",
        "A linked failed cash deposit is a personal contribution; choose Personal for its correction.",
    ),
    (
        "HEALTH_FAILED_DEPOSIT_UNAVAILABLE",
        "Choose this owner's actual failed cash deposit with the same destination, contribution date and designated year. Update or remove its correction before changing or deleting that original source.",
    ),
    (
        "HEALTH_FAILED_DEPOSIT_RETURN_ABOVE_DEPOSIT",
        "The combined principal returned for this original source, destination, deposit date and year cannot exceed its actual failed cash deposit. Other manual deposits provide separate correction room.",
    ),
    ("HEALTH_FAILED_DEPOSIT_CHOICE", "{0} — original failed cash deposits"),
    (
        "MEDICAL_DEPOSIT_YEAR_INVALID",
        "Use the actual ordinary contribution year accepted by the custodian for this deposit date.",
    ),
    (
        "MEDICAL_DEPOSIT_AFTER_DEADLINE",
        "The actual deposit must be no later than its applicable ordinary contribution deadline.",
    ),
    (
        "MEDICAL_REQUIRED_OUTPUT_UNAVAILABLE",
        "A required medical-account amount exceeds the report's supported amount range. The federal estimate is unavailable; retain the original account and source records.",
    ),
    (
        "MEDICAL_EXPLANATION_UNAVAILABLE",
        "A medical-account worksheet explanation exceeds the display amount range. The known income and tax remain included; retain the original account records.",
    ),
    (
        "ISSUE_ARCHER_MSA_TREATMENT_UNCONFIRMED",
        "This attempted medical-account rollover is not tax-free. Supply the actual custodian-accepted ordinary cash contribution year and deadline, or actual property character, before calculating its destination contribution and excess treatment. Do not enter the same deposit again in manual contribution totals.",
    ),
    (
        "ISSUE_ARCHER_MSA_DISABILITY_BEFORE_BIRTH",
        "The disability onset on a Form 1099-SA cannot precede the account holder's birth.",
    ),
    (
        "ISSUE_ARCHER_MSA_POST_DEATH_ACTIVITY",
        "A payment after the account holder's death needs the beneficiary's account or inheritance treatment.",
    ),
    (
        "ISSUE_INHERITED_MEDICAL_ESTATE_DATE_MISMATCH",
        "An estate-beneficiary account belongs on the original holder's final return; its death date must match People.",
    ),
    (
        "ISSUE_INHERITED_MEDICAL_BENEFICIARY_PREDECEASED",
        "An individual medical account beneficiary must survive the original holder; use estate treatment when the estate is the beneficiary.",
    ),
    (
        "NOTE_HSA_LAST_MONTH_RULE",
        "{0}: this HSA uses the 2026 last-month rule. Retain the monthly coverage and contribution records through December 31, 2027. A nonexcepted first loss in 2026 is included here; a first loss in 2027 belongs on the 2027 return. The amount above the ordinary monthly limit is {1,number,currency}.",
    ),
    (
        "ISSUE_AN_ALLOCATION_WITH_YOUR_SPOUSE_APPLIES_ONLY_TO_SEPARATE_RETURNS_FOR_E5EBCD38",
        "An allocation with your spouse applies only to separate returns (Form 8962 Part IV allocation situation 2).",
    ),
    (
        "ISSUE_AN_ELIGIBLE_2026_ARCHER_HDHP_MUST_MEET_THE_IRS_DEDUCTIBLE_AND_OUT_O_8BE1BDA9",
        "An eligible 2026 Archer HDHP must meet the IRS deductible and out-of-pocket limits; otherwise mark the month ineligible.",
    ),
    (
        "ISSUE_AN_OWNER_WITH_CONTRIBUTIONS_UNDER_THE_DECEMBER_FAMILY_METHOD_MUST_B_F7C5DF33",
        "An owner with contributions under the December family method must be eligible on December 1.",
    ),
    (
        "ISSUE_ARCHER_COMPENSATION_MUST_LINK_THIS_OWNER_S_W_2_FORM_8919_WAGES_OR_A_9C3E430C",
        "Archer compensation must link this owner's W-2, Form 8919 wages or a Schedule C or Schedule F business maintaining the HDHP.",
    ),
    (
        "ISSUE_ARCHER_MSA_CONTRIBUTIONS_EXCEED_COVERAGE_ALLOCATION_OR_LINKED_COMPE_C7A07770",
        "Archer MSA contributions exceed coverage/allocation or linked compensation limits, or employer deposits bar personal deductions; excess treatment is required.",
    ),
    (
        "ISSUE_A_DEATH_EXCEPTION_MUST_AGREE_WITH_THE_ACCOUNT_OWNER_S_FINAL_RETURN_F625155C",
        "A death exception must agree with the account owner's final-return death date.",
    ),
    (
        "HSA_OTHER_OWNER_ELIGIBILITY_REQUIRED",
        "Enter the other filer's actual monthly HSA eligibility, including an ineligible full year. An absent HSA account does not establish whether spouses share the family contribution limit.",
    ),
    (
        "HSA_LIMIT_METHOD_NEEDS_ACCOUNT",
        "Enter an actual HSA account before choosing the December contribution-limit method.",
    ),
    (
        "HSA_OTHER_OWNER_CONTEXT_INVALID",
        "Supply other-filer eligibility only for a married return with exactly one listed HSA owner, with all twelve months when eligibility changed. When both owners have entries, use those entries' eligibility instead.",
    ),
    (
        "HSA_DEPENDENCY_SOURCE_REQUIRED",
        "Confirm whether this HSA owner can be claimed as a dependent to calculate contribution income and excess tax. A qualified IRA-to-HSA funding transfer also requires an eligible owner; claimable owners can still receive ordinary HSA distributions.",
    ),
    (
        "ISSUE_BOTH_SPOUSES_HISTORICAL_RECORDS_MUST_IDENTIFY_THEIR_SHARED_FAMILY_A_786BF2A0",
        "Both spouses' historical records must identify their shared family allocation.",
    ),
    (
        "ISSUE_COMBINED_CONTRIBUTIONS_EXCEED_THE_MONTHLY_SHARED_FAMILY_LIMIT_AFTER_52D649B7",
        "Combined contributions exceed the monthly shared family limit after Archer MSA contributions; excess-contribution treatment is required.",
    ),
    (
        "ISSUE_CONFIRM_A_COMPLETE_TIMELY_ARCHER_MSA_CORRECTION_ITS_CONTRIBUTION_YE_68B7F8FA",
        "Confirm a complete timely Archer MSA correction, its contribution year, deadline and known employer wage reporting. Late or incomplete corrections need separate treatment.",
    ),
    (
        "ISSUE_CONFIRM_A_COMPLETE_TIMELY_HSA_CORRECTION_AND_CONTRIBUTION_YEAR_REPO_E7E27A00",
        "An HSA excess-contribution return made after its correction deadline needs separate treatment.",
    ),
    (
        "ISSUE_THE_PRIOR_ELIGIBILITY_HISTORIES_MUST_AGREE_ACROSS_OVERLAPPING_TESTING_PERIODS",
        "The prior last-month-rule history and each prior funding transfer must agree on the first loss of eligibility across overlapping testing periods.",
    ),
    (
        "ISSUE_CONFIRM_QUALIFIED_COVERAGE_ELIGIBLE_FAMILY_RELATIONSHIP_MEDICAL_ELI_40924044",
        "Confirm qualified coverage, eligible family relationship, medical eligibility and each month's employer long-term-care eligibility.",
    ),
    (
        "HSA_EXCESS_ACCOUNTING_REQUIRED",
        "Contributions exceed this owner's HSA contribution limit. Review eligibility, the family split and timely returns of excess contributions. If excess remains, open Excess contributions to this HSA and enter the year-end account value and any later deposits so the App can calculate Form 5329's 6% additional tax.",
    ),
    (
        "ISSUE_CURRENT_LAST_MONTH_TESTING_HISTORY_REQUIRES_DECEMBER_1_ELIGIBILITY_14EC49E7",
        "Current last-month testing history requires December 1 eligibility and a death exception consistent with People.",
    ),
    (
        "ISSUE_DECEMBER_BASED_ELIGIBILITY_CANNOT_BEGIN_AFTER_THE_ACCOUNT_OWNER_S_D_0DFC1873",
        "December-based eligibility cannot begin after the account owner's death.",
    ),
    (
        "ISSUE_DO_NOT_COUNT_ONE_COMPENSATION_SOURCE_TWICE_D840887D",
        "Do not count one compensation source twice.",
    ),
    (
        "ISSUE_ELIGIBILITY_LOSS_CANNOT_FIRST_OCCUR_AFTER_THE_ORIGINAL_ACCOUNT_OWNE_1C32D4D4",
        "Eligibility loss cannot first occur after the original account owner's death.",
    ),
    (
        "ISSUE_ELIGIBLE_ARCHER_MSA_MONTHS_CANNOT_FOLLOW_DEATH_OR_BELONG_TO_A_DEPEN_1C818CF3",
        "Eligible Archer MSA months cannot follow death or belong to a dependent.",
    ),
    (
        "ISSUE_EMPLOYER_DEPOSITS_MUST_BE_SUPPORTED_BY_COMPENSATION_FROM_THE_CONTRI_D2405C7C",
        "Employer deposits must be supported by compensation from the contributing employer, not by an unrelated self-employed business.",
    ),
    (
        "ISSUE_EMPLOYER_EXCESS_DEPOSITED_IN_2026_FOR_2025_CANNOT_EXCEED_THE_RETAIN_11C1C5AC",
        "Employer excess deposited in 2026 for 2025 cannot exceed the retained 2025 Form 5329 total excess.",
    ),
    (
        "ISSUE_FAMILY_ALLOCATION_REQUIRES_COMPLETE_SPOUSE_COVERAGE_AN_ELIGIBLE_FAM_CE8DC988",
        "Confirm the contribution-limit method and actual owner eligibility. December requires December1 eligibility; family capacity is divided only while both spouses are eligible. Use other-filer context only when that person has no HSA entry; separate returns retain only the taxpayer's account.",
    ),
    (
        "ISSUE_FUNDING_EXCEEDS_THE_ANNUAL_AMOUNT_FOR_THE_COVERAGE_AT_TRANSFER_AND_C6E212C2",
        "Funding exceeds the annual amount for the coverage at transfer and the owner's age-55 catch-up; both permitted transfers share the family maximum.",
    ),
    (
        "ISSUE_LINK_A_SAME_OWNER_IRA_DISTRIBUTION_AND_MATCHING_TRADITIONAL_OR_ROTH_2FFBD8E7",
        "Link a same-owner IRA distribution and matching traditional or Roth basis method. Roth source distributions cannot follow HSA receipt and need complete opening history. Basis funding needs dated aggregate balances. Earlier own nonqualified Roth withdrawals require confirmed opening history and distinct actual payment dates. Current regular contributions are supported only when each recorded Roth value covers its transfer plus all opening basis and retained annual regular contributions; other incoming basis or earlier activity needs further support. Inherited sources need the linked beneficiary/deceased-owner history and must follow the original owner's death. Nonqualified inherited Roth funding also supports dated ordinary withdrawals in that group; its opening regular and conversion basis remains separate from personal accounts and other deceased owners. A funding-first answer cannot override an entered earlier ordinary withdrawal date in that same Roth group.",
    ),
    (
        "ISSUE_LINK_THE_EMPLOYER_OR_BUSINESS_MAINTAINING_THE_HDHP_TO_CALCULATE_THE_9968DAAC",
        "Link the employer or business maintaining the HDHP to calculate the compensation limit.",
    ),
    (
        "ISSUE_OBTAIN_THE_APPLICABLE_SECOND_LOWEST_COST_SILVER_PLAN_PREMIUM_FROM_T_01337A24",
        "Obtain the applicable second-lowest-cost silver plan premium from the Marketplace or HealthCare.gov's tax tool before claiming credit for a paid coverage month (Form 8962 lines 11 to 23 column (b)).",
    ),
    (
        "ISSUE_POST_DEATH_CORRECTIVE_DISTRIBUTIONS_REQUIRE_THE_BENEFICIARY_S_ACCOU_54757DF0",
        "Post-death corrective distributions require the beneficiary's account or inheritance treatment.",
    ),
    (
        "ISSUE_POST_DEATH_EMPLOYER_DEPOSITS_NEED_BENEFICIARY_OR_ESTATE_TREATMENT_559CB33D",
        "Post-death employer deposits need beneficiary or estate treatment.",
    ),
    (
        "ISSUE_POST_DEATH_HSA_CORRECTIONS_REQUIRE_BENEFICIARY_OR_ESTATE_TREATMENT_0EE8EEC3",
        "Post-death HSA corrections require beneficiary or estate treatment.",
    ),
    (
        "ISSUE_POST_DEATH_HSA_WITHDRAWALS_REQUIRE_BENEFICIARY_ACCOUNT_TREATMENT_08CA97B0",
        "Post-death HSA withdrawals require beneficiary account treatment.",
    ),
    (
        "ISSUE_PRIOR_CONTRIBUTIONS_MUST_FIT_THE_2025_LIMIT_INCLUDING_THE_LAST_MONT_9539DED2",
        "Prior contributions must fit the 2025 limit including the last-month rule; remaining prior excess needs separate treatment.",
    ),
    (
        "ISSUE_PRIOR_LAST_MONTH_RULE_ELIGIBILITY_LOSS_AND_ANY_DEATH_EXCEPTION_MUST_E4F84B65",
        "Prior last-month-rule eligibility loss and any death exception must agree with the account owner's final-return date.",
    ),
    (
        "ISSUE_QUALIFIED_FUNDING_CANNOT_EXCEED_THE_LINKED_SOURCE_IRA_DISTRIBUTION_FDB93B9C",
        "Qualified funding cannot exceed the linked source IRA distribution. QCDs use only the remaining source dollars.",
    ),
    (
        "ISSUE_QUALIFIED_FUNDING_IS_ONCE_PER_LIFETIME_WITH_ONLY_A_SECOND_TRANSFER_ACB9D00B",
        "Qualified funding is once per lifetime, with only a second transfer in a later month of the same year after self-only changes to family coverage.",
    ),
    (
        "ISSUE_RESOLVE_THIS_PARTNERSHIP_S_REGULAR_SELF_EMPLOYMENT_EARNINGS_BEFORE_CB3167F1",
        "Resolve this partnership's regular self-employment earnings before its health deduction.",
    ),
    (
        "ISSUE_SEPARATE_RETURN_FAMILY_CONTRIBUTIONS_REQUIRE_THE_OTHER_SPOUSE_S_SHA_F78A5F4F",
        "Separate-return family contributions require the other spouse's shared-limit allocation.",
    ),
    (
        "ISSUE_SUPPLY_BOTH_SPOUSES_COVERAGE_RECORDS_INCLUDING_A_ZERO_CONTRIBUTION_B30F0C5A",
        "Supply both spouses' coverage records, including a zero-contribution row, to determine the family limit.",
    ),
    (
        "ISSUE_THE_PRIOR_FUNDING_TRANSFER_CANNOT_PRECEDE_THE_ACCOUNT_OWNER_S_BIRTH_B039F8EB",
        "The prior funding transfer cannot precede the account owner's birth.",
    ),
    (
        "ISSUE_THE_SUPPORTED_FORM_5329_PATH_REQUIRES_A_CURRENT_PERSONAL_EMPLOYER_E_A421EC88",
        "The supported Form 5329 path requires a current personal/employer excess or retained 2025 line 48 with positive line 49, reconciled W-2 wages for employer excess, and the statutory equal division or an actual agreement for shared eligible family months. Monthly limits are divided by 12 and rounded to cents before qualified IRA funding is subtracted.",
    ),
    (
        "ISSUE_THE_SUPPORTED_OWNER_SPECIFIC_FORM_5329_PART_VI_PATH_REQUIRES_CURREN_2A3F1B47",
        "The owner-specific Form 5329 Part VI path requires current uncorrected excess or retained positive-tax 2025 excess, reconciled W-2 wages and complete year-end account value.",
    ),
    (
        "ISSUE_THE_TWO_HISTORICAL_FAMILY_ALLOCATIONS_MUST_USE_MATCHING_SPOUSE_COVE_928CA8C8",
        "The two historical family allocations must use matching spouse coverage, Archer deposits and reciprocal agreed shares.",
    ),
    (
        "ISSUE_TIMELY_RETURNED_PERSONAL_AND_EMPLOYER_PRINCIPAL_CANNOT_EXCEED_THEIR_5DBC74C3",
        "Timely returned personal and employer principal cannot exceed their respective original excess above the Form 8853 deduction or employer limit.",
    ),
    (
        "ISSUE_UNCORRECTED_2025_DESIGNATED_EMPLOYER_EXCESS_DEPOSITED_IN_2026_CANNO_CD748793",
        "Uncorrected 2025-designated employer excess deposited in 2026 cannot exceed retained 2025 Form 5329 line 40 excess.",
    ),
    (
        "ISSUE_USE_CONSISTENT_FIRST_LOSS_FACTS_FOR_OVERLAPPING_CURRENT_FUNDING_PRI_3F5AFB4B",
        "Use consistent first-loss facts for overlapping current funding, prior last-month and current last-month testing periods.",
    ),
    (
        "ISSUE_USE_ONE_ARCHER_CONTRIBUTION_RECORD_PER_OWNER_WITH_A_SPOUSE_ROW_ONLY_89036B4A",
        "Use one Archer contribution record per owner, with a spouse row only when married.",
    ),
    (
        "ISSUE_USE_ONE_COMBINED_HSA_ENTRY_PER_ELIGIBLE_FILER_AND_CONFIRM_MONTHLY_E_ECDA078E",
        "Use one combined HSA entry per eligible filer and confirm monthly eligibility and ordinary account treatment.",
    ),
    (
        "ISSUE_A_LONG_TERM_CARE_PREMIUM_NAMES_SOMEONE_WHO_IS_NOT_ON_THIS_RETURN",
        "A long-term-care premium names someone who is not on this return. Choose the insured person from the people on the return.",
    ),
    (
        "MESSAGE_CHECK_THE_MONTHS_AND_AMOUNTS_OF_EACH_FORM_1095_A",
        "Check the months and amounts of each Form 1095-A.",
    ),
    ("DISPLAY_HEALTH_PLANS", "Business health and retirement plans"),
    ("DISPLAY_MARKETPLACE_PEOPLE", "People and tax families"),
    ("DISPLAY_MARKETPLACE_SPECIAL", "Special situations"),
    (
        "PTC_MARRIAGE_INELIGIBLE",
        "The alternative marriage calculation is not available from these eligibility facts. The regular reconciliation is retained.",
    ),
    (
        "PTC_TAX_FREE_PREMIUMS_ABOVE_NET_PAID",
        "Tax-free premium funding cannot exceed the actual net premium paid.",
    ),
    (
        "PTC_TAX_FREE_COVERAGE_ABOVE_TOTAL",
        "The coverage-family part cannot exceed the total actual tax-free premium funding.",
    ),
    (
        "HEALTH_POLICY_PLAN_MONTH_REQUIRED",
        "Enter the plan's actual establishment and employer-coverage eligibility for each assigned Marketplace month in Self-employed benefits. Enter zero ordinary premiums when the saved Form 1095-A is its only premium source; the policy assignment already supplies that payment.",
    ),
    (
        "HEALTH_POLICY_EXPENSE_ALLOCATION_REQUIRED",
        "Use the actual premium-funding records to identify how much tax-free payment or reimbursement applies to coverage-family premiums and how much applies to other enrolled people. The self-employed deduction for this policy needs that attribution; the App does not assume a person's reimbursement follows a silver-plan percentage.",
    ),
    (
        "HEALTH_POLICY_MEDICAL_ALLOCATION_REQUIRED",
        "This Marketplace policy includes someone whose premiums are not independently established as medical expenses for this return. Its medical deduction is omitted pending actual premium-cost and funding attribution. A silver-plan percentage or tax-family membership does not establish that cost; the known premium-credit reconciliation is retained.",
    ),
    (
        "HEALTH_POLICY_CHILD_FUNDING_ABOVE_COST",
        "The child-funded portion cannot exceed the actual included child cost or premium funding. Use the source's actual attribution, independently of the silver-plan allocation.",
    ),
    (
        "HEALTH_POLICY_BENCHMARK_REQUIRED",
        "Obtain the actual applicable silver-plan benchmarks for this policy's coverage-family members and its other enrolled family members in Self-employed benefits. They are needed to divide specified and nonspecified premiums without guessing a head-count share.",
    ),
    (
        "PTC_MARRIAGE_SOURCE_REQUIRED",
        "Complete the actual premarriage statement assignments and applicable alternative-family benchmarks to estimate the optional marriage reduction. The regular reconciliation is retained.",
    ),
    ("PTC_RECORD_CHOICE", "{0} (record {1})"),
    (
        "PTC_DEPENDENT_QSEHRA_NOTICE_REQUIRED",
        "Use the actual employer notice for this dependent employee. A W-2 entered for you or your spouse cannot establish the dependent's permitted benefit.",
    ),
    (
        "PTC_QSEHRA_BENEFIT_MISMATCH",
        "The monthly permitted benefits must add to the notice's actual annual current-year permitted benefit. Do not include prior-year carryover or actual reimbursements instead.",
    ),
    (
        "PTC_QSEHRA_SOCIAL_ABOVE_GROSS",
        "The taxable Social Security component cannot exceed gross Social Security.",
    ),
    (
        "PTC_QSEHRA_EMPLOYEE_RETURN_REQUIRED",
        "Enter this person's actual required return on Marketplace before selecting it as this QSEHRA income source, or use documentary components.",
    ),
    (
        "PTC_QSEHRA_RETURN_CONFLICT",
        "These documentary components differ from this person's required return already entered. Correct that source or select the existing required return; do not keep conflicting income amounts.",
    ),
    (
        "PTC_QSEHRA_RELATION_INVALID",
        "Identify the employee's actual spouse and dependents once using existing People identities; do not repeat the employee, spouse or a dependent.",
    ),
    (
        "PTC_DEPENDENT_QSEHRA_RECONCILIATION_REQUIRED",
        "A QSEHRA provided to a person you claim needs that employee's own affordability and family relationships. The App cannot substitute your whole-return family bar or omit the permitted-benefit reduction.",
    ),
    (
        "PTC_SOURCE_FAMILY_REQUIRED",
        "Identify this covered person's final tax family before reconciling the policy.",
    ),
    (
        "PTC_SOURCE_ATTESTER_REQUIRED",
        "Identify the taxpayer who attested that this person would be in their tax family. The advance credit still needs reconciliation.",
    ),
    (
        "PTC_SOURCE_ELIGIBILITY_REQUIRED",
        "Complete this covered person's actual eligibility for the month before reconciling the policy.",
    ),
    (
        "PTC_SOURCE_ENROLLMENT_REQUIRED",
        "Complete the actual covered people and applicable enrollment dates for this policy month.",
    ),
    (
        "PTC_SOURCE_ALLOCATION_REQUIRED",
        "Complete this shared policy's applicable family allocation and agreements before reconciling its advance credit.",
    ),
    (
        "PTC_SOURCE_LAWFUL_PREMIUM_REQUIRED",
        "Obtain the actual premium excluding people who were not lawfully present, or an authorized reference month with the same lawful covered people.",
    ),
    (
        "PTC_SOURCE_LAWFUL_PAYMENT_REQUIRED",
        "Obtain the actual net premium paid for lawful coverage when partial payments or refunds affected this month. A lawful enrollment cost does not establish how those payments were allocated.",
    ),
    (
        "PTC_SOURCE_BENCHMARK_REQUIRED",
        "Obtain the applicable coverage-family benchmark for this month. Repeated statement benchmarks must not be added together.",
    ),
    (
        "PTC_SOURCE_PAYMENT_REQUIRED",
        "Complete the actual premium-payment and coverage facts, including any advance returned by the insurer.",
    ),
    (
        "PTC_AGREED_SHARES_ABOVE_WHOLE",
        "The agreed shares across these tax families cannot exceed the whole policy.",
    ),
    (
        "PTC_EXPLICIT_ENROLLER_SHARE_MISMATCH",
        "When every listed family agrees and the enrolling family is listed, the shares must add to the whole policy.",
    ),
    ("PTC_SHARE_EXCEEDS_WHOLE", "An allocation share cannot exceed the whole amount."),
    ("PTC_MONTHS_REVERSED", "The last allocation month cannot precede the first month."),
    ("PTC_COVERAGE_DATES_REVERSED", "Coverage cannot end before it begins."),
    (
        "MESSAGE_SPLIT_THE_FAMILY_CONTRIBUTION_LIMIT_ONLY_WHEN_BOTH_OF_YOU_HAVE_AN_ACCOUNT_WI",
        "Split the family contribution limit only for overlapping eligible months when both of you have an account and either policy provides family coverage.",
    ),
    (
        "MESSAGE_THIS_RETURN_S_BUSINESSES_CHANGED_REOPEN_THE_EDITOR_BEFORE_TRYING_AGAIN",
        "This return's businesses changed. Reopen the editor before trying again.",
    ),
    (
        "MESSAGE_YOUR_SPOUSE_S_ACCOUNT_BELONGS_ON_THIS_RETURN_ONLY_WHEN_YOU_FILE_JOINTLY",
        "Your spouse's account belongs on this return only when you file jointly.",
    ),
    (
        "PROBLEM_INVALID_HEALTH_PLAN_SOURCE",
        "Check each health plan's actual establishment, same-owner wage and insured people. For partial entity reimbursement, enter actual funded portions of ordinary, child and LTC premiums and assigned Marketplace payments; their total must match the existing reimbursement.",
    ),
    (
        "PROBLEM_INVALID_HEALTH_POLICY_ASSIGNMENT",
        "Reopen Self-employed benefits and select saved Marketplace statements and current businesses. Each policy, month and business assignment must be unique, with positive allocation parts and valid benchmark sources.",
    ),
    (
        "PROBLEM_UNDISPLAYABLE_BUSINESS_BENEFIT_RECORD",
        "This return holds a business benefit record this page cannot show yet. Use a fresh development task.",
    ),
    (
        "PARTNER_PREMIUMS_BELOW_K1",
        "The premiums entered must include K-1 box 13 code M and the part charged against your distributions.",
    ),
    (
        "PARTNER_DISTRIBUTION_PREMIUMS_ABOVE_19A",
        "The part charged against your distributions cannot exceed box 19 code A cash distributions.",
    ),
    (
        "PARTNER_BOX_13R_NEEDS_PLAN",
        "K-1 box 13 code R shows a retirement plan payment: choose its plan.",
    ),
    (
        "PARTNER_PLAN_WITHOUT_BOX_13R",
        "A plan contribution needs its amount in K-1 box 13 code R on the Schedule K-1 topic.",
    ),
    ("MARKETPLACE_ADVANCE_ABOVE_PREMIUM", "Column C cannot exceed column A for the same month."),
    (
        "MARKETPLACE_TAXABLE_SOCIAL_ABOVE_GROSS",
        "The taxable part of Social Security benefits (line 6b) cannot exceed the benefits (line 6a).",
    ),
    (
        "HSA_ACCOUNT_TWICE",
        "Enter one account for each owner; combine all of an owner's HSAs into that account.",
    ),
    (
        "HSA_FAMILY_PARTS_ZERO",
        "At least one of you must have a nonzero share of the HSA family contribution limit.",
    ),
    ("REVIEW_MARKETPLACE_COUNT", "{0,plural,one {# Form 1095-A} other {# Forms 1095-A}}"),
    (
        "REVIEW_MARKETPLACE_CREDIT",
        "{0,plural,one {# Form 1095-A} other {# Forms 1095-A}} · credit {1,number,currency}",
    ),
    (
        "REVIEW_MARKETPLACE_REPAY",
        "{0,plural,one {# Form 1095-A} other {# Forms 1095-A}} · repay {1,number,currency}",
    ),
];
