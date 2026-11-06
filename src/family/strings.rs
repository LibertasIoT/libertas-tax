pub(crate) const FAMILY_STRINGS: &[(&str, &str)] = &[
    (
        "NOTE_CARE_PROVIDER_DETAILS",
        "Form 2441 Part I: {0}, {1}; identifying-number entry: {2}. The App never collects the number itself.",
    ),
    ("CARE_PROVIDER_INDIVIDUAL", "SSN or ITIN held"),
    ("CARE_PROVIDER_ORGANIZATION", "EIN held"),
    ("CARE_PROVIDER_TAX_EXEMPT", "Tax-Exempt"),
    ("CARE_PROVIDER_SEE_W2", "See W-2"),
    ("CARE_PROVIDER_LAFCP", "LAFCP"),
    ("CARE_PROVIDER_REFUSED", "See Attached Statement"),
    ("CARE_PROVIDER_NOT_OBTAINED", "not obtained"),
    ("CARE_INVALID_RECORD", "Check the care amounts, owners and provider details."),
    (
        "CARE_WAGE_SOURCE_INVALID",
        "Keep exactly one benefit answer for each current W-2 with box 10. Change the source on Wages.",
    ),
    (
        "CARE_FILER_SOURCE_INVALID",
        "Choose a joint filer whose own page says student or unable to care for themselves.",
    ),
    (
        "CARE_BUSINESS_SOURCE_INVALID",
        "Choose an existing business, farm or partnership K-1 of this owner.",
    ),
    (
        "CARE_FILING_STATUS_INVALID",
        "Reopen Child and dependent care after changing your filing status.",
    ),
    (
        "CARE_BENEFIT_PARTS_ABOVE_BOX_10",
        "The part in box 1 plus forfeited benefits cannot exceed box 10.",
    ),
    ("CARE_STUDENT_NEEDS_FIVE_MONTHS", "A full-time student needs at least five calendar months."),
    (
        "CARE_PARENT_PROVIDER_NEEDS_A_CHILD",
        "The other-parent relationship requires a qualifying child under 13.",
    ),
    (
        "CARE_PLAN_MAXIMUM_REQUIRED",
        "For dependent-care benefits, enter the combined maximum of the distinct plans. With no benefits or grace balance, choose no benefits.",
    ),
    (
        "CARE_COSTS_NEED_A_PROVIDER",
        "Enter the provider for the care costs, including identification or due diligence.",
    ),
    ("PRIOR_CARE_LINE_6_ABOVE_LINE_3", "2025 Form 2441 line 6 cannot exceed line 3."),
    (
        "WARN_CARE_RELATED_PROVIDER_LEFT_OUT",
        "Care paid to {0} was left out because this relationship does not qualify.",
    ),
    (
        "WARN_CARE_PROVIDER_NOT_IDENTIFIED_LEFT_OUT",
        "Care paid to {0} was left out because provider identification or due diligence was not obtained.",
    ),
    (
        "WARN_CARE_DEEMED_INCOME_MONTHLY",
        "The selected months and combined earnings give a conservative earned-income limit. Shared months are assigned to one spouse; keep monthly records if a larger allowance applies.",
    ),
    (
        "NOTE_CARE_PROVIDER_STATEMENT_ATTACH",
        "Attach the due-diligence statement for {0}, who refused identification.",
    ),
    (
        "NOTE_CARE_HOUSEHOLD_EMPLOYEE_SCHEDULE_H",
        "Check household employment taxes on Other taxes (Schedule H).",
    ),
    (
        "NOTE_CARE_WORKSHEET_A_ATTACH",
        "Attach Worksheet A with the prior-year provider and qualifying person details.",
    ),
    ("DISPLAY_CARE_PEOPLE", "People and providers"),
    ("DISPLAY_CARE_BENEFITS", "Employer benefits"),
    ("DISPLAY_CARE_OTHER", "Filer and prior-year facts"),
    ("CARE_MONTH_1", "January"),
    ("CARE_MONTH_2", "February"),
    ("CARE_MONTH_3", "March"),
    ("CARE_MONTH_4", "April"),
    ("CARE_MONTH_5", "May"),
    ("CARE_MONTH_6", "June"),
    ("CARE_MONTH_7", "July"),
    ("CARE_MONTH_8", "August"),
    ("CARE_MONTH_9", "September"),
    ("CARE_MONTH_10", "October"),
    ("CARE_MONTH_11", "November"),
    ("CARE_MONTH_12", "December"),
    ("DISPLAY_DEPENDENT_ABOUT", "About this person"),
    ("DISPLAY_DEPENDENT_HOME", "Home, support and claims"),
    (
        "ISSUE_COVERDELL_CORRECTION_REVIEW",
        "A recorded Coverdell correction no longer agrees with its linked income source. Reconcile the actual principal, earnings and beneficiary before its additional tax is computed.",
    ),
    (
        "COVERDELL_BENEFICIARY_TWICE",
        "Enter each Coverdell beneficiary once, including all their accounts.",
    ),
    (
        "COVERDELL_DONOR_TWICE",
        "Enter each contributor once for this beneficiary, with that contributor’s deposits from every account.",
    ),
    (
        "COVERDELL_LOSS_ABOVE_PRINCIPAL",
        "Returned principal plus its associated net income cannot be negative.",
    ),
    (
        "COVERDELL_OWNER_NOT_ON_RETURN",
        "A beneficiary or on-return contributor must be a filer on this return. Enter another individual through their own contributor facts.",
    ),
    (
        "COVERDELL_RETURN_ABOVE_CONTRIBUTIONS",
        "Returned principal exceeds the contributions to that account available on the correction date.",
    ),
    ("COVERDELL_CORRECTION_SOURCE_TWICE", "Link each returned-excess Form 1099-Q only once."),
    (
        "COVERDELL_CORRECTION_SOURCE_INVALID",
        "Choose the same beneficiary’s current-year Coverdell code 2 source with matching principal, earnings and actual correction year.",
    ),
    (
        "ISSUE_COVERDELL_ELIGIBILITY",
        "A contribution after age 18 without an established special-needs exception needs the actual account consequences under section 530; the App cannot assume a permitted account or a zero added tax.",
    ),
    (
        "ISSUE_COVERDELL_LIMIT_UNAVAILABLE",
        "The actual contributor limit records do not establish the beneficiary’s section 4973(e) tax and next-year excess. No donor or contribution limit is invented.",
    ),
    (
        "NOTE_COVERDELL_LIMIT_ENDPOINTS",
        "The missing contributor limit cannot change this beneficiary’s tax or next-year excess: both possible limit endpoints give the same result. Unresolved intermediate room is not printed as a known amount.",
    ),
    (
        "QA1099_BOXES_DO_NOT_ADD_UP",
        "The earnings (box 2) plus the basis (box 3) must equal the distribution (box 1).",
    ),
    ("Q1099_BENEFICIARY_BOX6_CONFLICT", "Reconcile the student with the printed box 6 check."),
    ("QA1099_TREATMENT_NEEDED", "Answer what this ABLE payment was."),
    (
        "QA1099_ROLLOVER_EARNINGS_NOT_RECONCILED",
        "Reconcile the rollover earnings and the remaining distribution.",
    ),
    (
        "QA1099_ROLLOVER_ABOVE_GROSS",
        "The amount put back cannot be more than the distribution (box 1).",
    ),
    (
        "QA1099_ROLLOVER_EARNINGS_ABOVE_AMOUNT",
        "The earnings in the amount put back cannot be more than that amount.",
    ),
    (
        "NOTE_COVERDELL_PRIOR_ZERO_TAX",
        "The retained Coverdell excess carries under section 4973(e) even though the prior tax was zero. The 2026 draft Form 5329 instructions condition line 26 on prior tax; this estimate follows the Code and retains the actual excess.",
    ),
    (
        "WARN_ADOPTION_CHILD_LEFT_OUT",
        "{0}: this child's eligibility or expenses aren't confirmed, so no adoption credit or employer-benefit exclusion is figured for this child; your tax may be lower.",
    ),
    (
        "WARN_ADOPTION_EXCLUSION_SPLIT_LEFT_OUT",
        "{0}: the split of a partly available employer-benefit exclusion between the two employees isn't confirmed, so that exclusion is left out and all the code T benefits are taxable; your tax may be lower.",
    ),
    (
        "WARN_ADOPTION_BENEFITS_LEFT_OUT",
        "{0}: these employer adoption benefits' plan or records aren't supported, so no exclusion is figured and they are taxable; your tax may be lower.",
    ),
    (
        "WARN_ADOPTION_SHARED_LIMIT_LEFT_OUT",
        "Adoption limits shared with another return aren't confirmed as agreed, so no credit or exclusion is figured for that child; your tax may be lower.",
    ),
    (
        "ISSUE_ADOPTION_CODE_T_ABOVE_W_2",
        "The adoption benefits entered for this employee are more than the box 12 code T amounts on the W-2s: correct the W-2 or the adoption benefits.",
    ),
    (
        "WARN_W2_CODE_T_TAXABLE_NO_ADOPTION_RECORD",
        "{0}: its box 12 code T adoption benefits are not entered under an adoption on More credits, so they are taxable on Form 1040 line 1f; enter the adoption to exclude them, or your tax may be lower.",
    ),
    (
        "SCHOLARSHIP_INCLUDED_ABOVE_BOX_5",
        "The part of the grant the student includes in income cannot be more than the box 5 grant.",
    ),
    ("DEATH_DATE_OUTSIDE_2026", "Enter a date of death in 2026."),
    ("DEATH_BEFORE_BIRTH", "The date of death can't be before the date of birth."),
    (
        "NOTE_INCOME_IN_RESPECT_OF_A_DECEDENT",
        "{0}: {1,number,currency} paid after {2}'s death is income in respect of a decedent; it belongs on the estate's Form 1041 or the recipient's own return, not this final return (section 691(a), Pub. 559).",
    ),
    (
        "WARN_AFTER_DEATH_SPLIT_LEFT_OUT",
        "{0}: this statement also prints other amounts, so the part paid after the death isn't split off; it is calculated as printed and your tax may be higher.",
    ),
    (
        "WHOLE_RETURN_RESIDENT_CHOICE_OR_DUAL_STATUS",
        "Choose the resident choice for a nonresident spouse or a dual-status or territory return, not both: a dual-status return can't use the choice, and the choice makes both of you residents for the whole year.",
    ),
    (
        "RESIDENCE_ANSWER_NOT_ALLOWED",
        "On the final return of a person who died, a person's time in your home is counted in days through the date of death; on any other return it is counted in months.",
    ),
    (
        "RESIDENCE_DAYS_ABOVE_DAYS_ALIVE",
        "A person can't have lived in your home for more days than you were alive in 2026.",
    ),
    (
        "PERSON_CREDITS_CHILD_EIC",
        "You qualify for the child tax credit for {0}, up to {1,number,currency}, and {0} is a qualifying child for your earned income credit.",
    ),
    (
        "PERSON_CREDITS_CHILD",
        "You qualify for the child tax credit for {0}, up to {1,number,currency}.",
    ),
    (
        "PERSON_CREDITS_OTHER_EIC",
        "You qualify for the credit for other dependents for {0}, up to {1,number,currency}, and {0} is a qualifying child for your earned income credit.",
    ),
    (
        "PERSON_CREDITS_OTHER",
        "You qualify for the credit for other dependents for {0}, up to {1,number,currency}.",
    ),
    (
        "PERSON_CREDITS_EIC",
        "{0} is a qualifying child for your earned income credit, but gives you no child tax credit or credit for other dependents.",
    ),
    (
        "PERSON_CREDITS_NONE",
        "Based on these answers, {0} gives you no child tax credit, credit for other dependents or earned income credit.",
    ),
    (
        "SECTION179_SPOUSE_SOURCE_INVALID",
        "Check your spouse's qualifying section 179 purchase cost and allocation. The cost must be nonnegative and within the supported amount, and each elected percentage must be from 0% through 100%.",
    ),
    ("LABEL_RETURN", "{0,number,0} federal return"),
    ("REVIEW_FILING_NAME_SINGLE", "Single"),
    ("REVIEW_FILING_NAME_JOINT", "Married filing jointly"),
    ("REVIEW_FILING_NAME_SEPARATE", "Married filing separately"),
    ("REVIEW_FILING_NAME_HEAD", "Head of household"),
    ("REVIEW_FILING_NAME_SURVIVING", "Qualifying surviving spouse"),
    (
        "NOTE_TRUMP_ACCOUNT_LATE_EXCESS_TAXPAYER",
        "Trump account — taxpayer: {0,number,currency} of excess principal was returned after the 2026-return deadline. It does not reduce 2026 Form 5329 line 60 or the 6% tax; review the receipt-year distribution and earnings separately.",
    ),
    (
        "NOTE_TRUMP_ACCOUNT_LATE_EXCESS_SPOUSE",
        "Trump account — spouse: {0,number,currency} of excess principal was returned after the 2026-return deadline. It does not reduce 2026 Form 5329 line 60 or the 6% tax; review the receipt-year distribution and earnings separately.",
    ),
    ("DISPLAY_ALIMONY_AGREEMENT", "Agreement"),
    ("DISPLAY_ALIMONY_PAYMENTS", "Payments"),
    ("DISPLAY_EDUCATION_COLLEGE", "Student and college costs"),
    ("DISPLAY_EDUCATION_K12", "K–12 and training"),
    ("DISPLAY_EDUCATION_LOANS", "Student loans"),
    (
        "NOTE_ADDITIONAL_ABLE_ACCOUNT_RETURN_YEAR_CONTRIBUTIONS_ACCOUNT_VALUES_AND_DI_24653906",
        "Additional ABLE account: 2026 contributions, account values and distributions must include both accounts once. Unless the additional account is fully corrected or consolidated by the beneficiary's return deadline, including extensions, it ceases to be an ABLE account immediately after that deadline; review the following year separately.",
    ),
    (
        "NOTE_ADOPTION_SEPARATE_RETURN_EXCEPTION_NOT_MET",
        "Adoption — {0}: current credit and employer-benefit exclusion are not allowed on this separate return because the living-apart, home-cost or child-residence test is not met. Eligible prior credit carryforwards are evaluated separately.",
    ),
    (
        "ISSUE_AN_AGREEMENT_CHANGED_DURING_RETURN_YEAR_TO_THE_POST_2018_RULES_OR_REPLACED_F80A1653",
        "An agreement changed during 2026 to the post-2018 rules, or replaced during 2026 by a divorce decree signed after 2018, splits this year's payments at the change (Pub. 504): Schedule 1 line 2a or 19a for such a year is not calculated yet.",
    ),
    (
        "ISSUE_A_FOREIGN_ADOPTION_NOT_FINAL_BEFORE_RETURN_YEAR_CANNOT_HAVE_A_PRIOR_EXCLUS_1723F10D",
        "A foreign adoption not final before 2026 cannot have a prior exclusion-limit claim; prior taxable benefits need the foreign-adoption worksheet.",
    ),
    (
        "ISSUE_ABLE_PAYMENT_TO_NON_BENEFICIARY",
        "Form 1099-QA box 6 says this payment went to someone other than the designated beneficiary, so it is not figured as your own withdrawal. Answer Death or Returned excess contribution if either fits; the additional taxes on a payment to another recipient are not estimated.",
    ),
    (
        "ISSUE_ABLE_BENEFICIARY_NOT_ON_RETURN",
        "An ABLE beneficiary other than you is on this return only when you file jointly.",
    ),
    (
        "ISSUE_CONFIRM_DISJOINT_CORRECTED_W_2_CODE_T_SOURCES_QUALIFIED_PLANS_NO_PR_6D13D308",
        "Confirm disjoint corrected W-2 code T sources, qualified plans and a joint return for domestic children; benefits for a foreign child paid to both spouses, and foreign benefits taxed before finalization, are not calculated. Deemed line 29 excess reduces joint line 1f, not either employee's earnings.",
    ),
    (
        "ISSUE_CONFIRM_ORDINARY_QUALIFYING_ALIMONY_AND_RESOLVE_RECAPTURE_PRE_1985_DABB0528",
        "Alimony under an instrument executed before 1985 follows special rules that are not calculated.",
    ),
    (
        "ISSUE_CONFIRM_THE_QUALIFIED_PLAN_EMPLOYEE_OWNER_NONDUPLICATED_BENEFITS_AN_291CE261",
        "Your spouse's employer adoption benefits need a joint return. Multiple employees for one child need additional coordination.",
    ),
    (
        "ISSUE_CONFIRM_THE_THIRD_YEAR_PAYMENT_HISTORY_AND_EXCEPTION_PRE_1987_INSTR_F7A45E5F",
        "Third-year recapture for a pre-1987 instrument, or one changed to the post-2018 rules during the recapture period, needs separate review.",
    ),
    (
        "ISSUE_CORRECTIVE_PRINCIPAL_CANNOT_EXCEED_THIS_BENEFICIARY_S_NONEXEMPT_202_9A2F2791",
        "Corrective principal cannot exceed this beneficiary's nonexempt 2026 contributions above the $5,000 limit.",
    ),
    (
        "ISSUE_EARLIER_TAXABLE_FOREIGN_BENEFITS_BELONG_TO_A_PENDING_FOREIGN_ADOPTI_1D3323FF",
        "Earlier taxable foreign benefits belong to a pending foreign adoption or one finalized in 2026; earlier finalizations require the previously filed exclusion history.",
    ),
    (
        "ISSUE_USE_THE_ACCOUNT_BENEFICIARY_S_OWN_RETURN_YEAR_RETURN_CONFIRM_THEY_WOULD_BE_11017AE6",
        "Use the account beneficiary's own 2026 return, confirm they would be under age 18 at year-end, and enter complete qualified-account and excess-distribution records. A deceased beneficiary needs final-return facts, with all corrections before death and any death-date income reported separately on the proper return. A distribution after the 2026-return deadline does not reduce this year's excess.",
    ),
    (
        "MESSAGE_SAVE_YOUR_OWN_FILER_PAGE_BEFORE_YOUR_SPOUSE_S",
        "Save your own filer page before your spouse's.",
    ),
    (
        "MESSAGE_YOUR_PARENT_S_RETURN_ENTER_THE_AMOUNTS_WITHIN_THE_SUPPORTED_RANGE_EACH_BROTH",
        "Your parent's return: enter the amounts within the supported range, each brother's or sister's name once.",
    ),
    ("PARENT_CHILD_TWICE", "Enter each brother or sister once."),
    (
        "PARENT_CHILD_GAINS_ABOVE_NET",
        "The qualified dividends and capital gain included cannot exceed that child's net unearned income.",
    ),
    (
        "PARENT_CHILD_GAIN_ABOVE_SOURCE",
        "The capital gain included cannot exceed that child's net capital gain before allocation.",
    ),
    (
        "INJURED_PRIOR_CARE_EXPENSE_TOTAL",
        "The actual expenses for the prior-year qualifying people must total the retained worksheet’s expenses paid in 2026.",
    ),
    (
        "SCHOLARSHIP_SPENDING_ABOVE_AWARD",
        "The restricted part, the part paying for required services and the part spent on qualified expenses cannot add up to more than the award.",
    ),
    (
        "HOH_NEEDS_HOME_COST",
        "Head of household requires paying more than half the cost of keeping up your home.",
    ),
    (
        "QSS_NEEDS_HOME_COST",
        "Qualifying surviving spouse requires paying more than half the cost of keeping up your home.",
    ),
    (
        "SUPPORTED_NEEDS_YOUR_SUPPORT",
        "Another person you support needs you to have paid more than half their support, a multiple support agreement (Form 2120), or, for your own child, the other parent's release (Form 8332).",
    ),
    (
        "CUSTODY_RELEASE_ONE_WAY",
        "Choose one direction for the Form 8332 release: you released this child to the other parent, or the other parent released them to you.",
    ),
    (
        "CUSTODY_RELEASE_NEEDS_YOUR_CHILD",
        "A Form 8332 release applies only to your own or adopted child: choose Child as the relationship.",
    ),
    (
        "ADOPTED_NEEDS_CHILD_ALL_YEAR",
        "The adopted-child exception applies only to your own or adopted child who lived with you all year: choose Child as the relationship and All year.",
    ),
    (
        "CUSTODY_RELEASE_TO_YOU_NEEDS_FEWER_NIGHTS",
        "A release from the other parent applies when the child lived with you for fewer nights: choose Part of the year with 6 months or fewer.",
    ),
    (
        "MULTIPLE_SUPPORT_NEEDS_SHARED_SUPPORT",
        "A multiple support agreement applies to another person you support when no one paid more than half their support: choose Another person you support and answer No to paying more than half.",
    ),
    (
        "MULTIPLE_SUPPORT_NOT_WITH_CUSTODY_RELEASE",
        "A Form 8332 release does not apply when a multiple support agreement governs the child's support: choose one.",
    ),
    (
        "OWN_SUPPORT_NOT_WITH_YOUR_SUPPORT",
        "You and this person cannot each pay more than half of their support: untick that they paid more than half of their own, or answer No to paying more than half yourself.",
    ),
    (
        "OWN_SUPPORT_NOT_WITH_SHARED_SUPPORT",
        "A Form 8332 release or a multiple support agreement needs parents or several supporters to have paid more than half the support, so it does not apply when this person paid more than half of their own: choose one.",
    ),
    (
        "DEPENDENT_AGE_SITUATION",
        "A child under 19 must be born on or after January 1, 2008, and a full-time student under 24 on or after January 1, 2003.",
    ),
    (
        "ANOTHERS_CHILD_NEEDS_SUPPORTED",
        "Being someone else's qualifying child applies only to another person you support: choose that for this person, or untick it.",
    ),
    (
        "DEPENDENT_OLDER_THAN_YOU",
        "A child under 19 or a student must be younger than you, or than your spouse on a joint return.",
    ),
    (
        "CARE_NEEDS_UNDER_13_OR_SELF_CARE",
        "Care for someone 13 or older counts only when they are unable to care for themselves.",
    ),
    (
        "TRUMP_EXEMPT_ABOVE_CONTRIBUTIONS",
        "The contributions exempt from the $5,000 limit cannot be more than the total contributions accepted.",
    ),
    (
        "TRUMP_LOSS_ABOVE_PRINCIPAL",
        "A loss on a returned excess contribution cannot be more than the principal returned.",
    ),
    (
        "TRUMP_ACCOUNT_TWICE",
        "Enter one Trump account entry for each account beneficiary on this return.",
    ),
    (
        "WHOLE_RETURN_NIIT_NEEDS_THE_RESIDENT_CHOICE",
        "The net investment income tax items apply only with the choice to treat your nonresident spouse as a resident.",
    ),
    // Form 8814: a parent's election to report a child's interest and dividends.
    (
        "WARN_CHILD_ELECTION_INCOME_TOO_HIGH_LEFT_OUT",
        "{0}: a child with $13,500 or more of interest and dividends cannot be reported on your return (Form 8814), so this child's income is left out of this estimate and the child must file their own 2026 return.",
    ),
    (
        "WARN_CHILD_ELECTION_WITHHOLDING_LEFT_OUT",
        "{0}: a child whose interest or dividends had federal income tax withheld cannot be reported on your return (Form 8814), so this child's income is left out of this estimate; the child files their own 2026 return to recover the withholding.",
    ),
    (
        "WARN_CHILD_ELECTION_SPECIAL_LEFT_OUT",
        "{0}: a child's statement with bond discount or premium treatment, a savings-bond exclusion, treaty treatment, section 1202 gain, foreign or other special treatment, or amounts received as a nominee isn't supported on your return (Form 8814), so this child's income is left out of this estimate and the child files their own 2026 return.",
    ),
    (
        "WARN_CHILD_ELECTION_INELIGIBLE_LEFT_OUT",
        "{0}: a child with other income, with estimated tax paid under their own number, or who is no longer one the return can report cannot be reported on your return (Form 8814), so this child's income is left out of this estimate and the child files their own 2026 return.",
    ),
    (
        "WARN_CHILD_ELECTION_NONCUSTODIAL_LEFT_OUT",
        "{0}: only the parent the child lived with for more than half of 2026 can report the child's interest and dividends (Form 8814), so this child's income is left out of this estimate and the child files their own 2026 return.",
    ),
    (
        "WARN_CHILD_ELECTION_OTHER_PARENT_LEFT_OUT",
        "{0}: the other parent's return, with the higher taxable income, reports this child's interest and dividends (Form 8814), so the child is left out of this estimate.",
    ),
    (
        "WARN_CHILD_ELECTION_PARENT_ASSUMED",
        "Assumed the other parent's return reports the interest and dividends of {0} (Form 8814), so this child is left out of this estimate; if your return reports them, choose that on the Interest and dividends page.",
    ),
    // Form 1099-Q, 529 plans and Coverdell ESAs (education_account.rs).
    (
        "Q1099_BOXES_DO_NOT_ADD_UP",
        "Box 2 earnings and box 3 basis must add up to the box 1 gross distribution.",
    ),
    (
        "Q1099_PART_ABOVE_GROSS",
        "The part put back and the part that does not qualify can't be more than the gross distribution.",
    ),
    (
        "Q1099_ROTH_ROLLOVER_NOT_529",
        "Only a 529 plan can transfer to a Roth IRA (box 4b). Choose the 529 plan account, or a different kind of transfer.",
    ),
    (
        "Q1099_ROTH_ROLLOVER_NOT_STUDENT",
        "A 529-to-Roth IRA transfer goes to the Roth IRA of the 529 account's beneficiary, who is the person this form is addressed to. Choose that the form is addressed to the student.",
    ),
    ("Q1099_ROTH_ROLLOVER_LOSS", "A 529-to-Roth IRA transfer can't have a loss in box 2."),
    (
        "Q1099_CODE_NOT_COVERDELL",
        "Box 7 codes 2, 3 and 6 apply only to a Coverdell ESA. Choose the Coverdell account, or a different box 7 code.",
    ),
    (
        "Q1099_EXCESS_NEEDS_BOXES",
        "A returned excess contribution shows its earnings in box 2. Enter boxes 2 and 3 instead of a year-end value.",
    ),
    (
        "ISSUE_Q1099_STUDENT_IS_THE_RECIPIENT",
        "This Form 1099-Q is addressed to the same person you chose as its student. Choose that the form is addressed to the student (box 6 not checked), or choose the other person.",
    ),
    (
        "ISSUE_EDUCATION_ACCOUNT_STUDENT_HAS_NO_EDUCATION_CREDIT_ENTRY",
        "A 529 or Coverdell student's college costs come from an education credit entry, but this person has none. Add the entry on Education credits or enter the costs on the 529 and Coverdell expenses record.",
    ),
    (
        "ISSUE_EDUCATION_ACCOUNT_EXPENSES_ENTERED_TWICE",
        "This person's college costs are entered on Education credits and again on the 529 and Coverdell expenses record. Enter them once: choose the education credit entry here, or remove the credit entry.",
    ),
    (
        "WARN_EDUCATION_ACCOUNT_EXPENSES_ASSUMED_NONE",
        "{0}: no costs are entered for this student, so all of this withdrawal's earnings are taxed. Add the student's costs under 529 and Coverdell expenses if any were paid.",
    ),
    (
        "NOTE_EDUCATION_ACCOUNT_CREDIT_EXPENSES",
        "An education credit entry uses college costs of a student with a 529 or Coverdell withdrawal, but its credit is zero, so those costs are not available to make the earnings tax-free. Remove the education credit entry to free the costs for the withdrawal.",
    ),
    (
        "NOTE_COVERDELL_EXCESS_PRIOR_RETURN",
        "A Coverdell excess contribution returned with box 7 code 3 was taxable on your 2025 return, so nothing is added to this return. If that return left out its earnings, it needs an amendment.",
    ),
    ("DISPLAY_ADOPT_CHILD", "Child"),
    ("DISPLAY_ADOPT_EXPENSES", "Expenses and prior credit"),
    ("DISPLAY_ADOPT_BENEFITS", "Benefits and wages"),
    ("DISPLAY_PARENT_RETURN", "Parent's return"),
    ("DISPLAY_PARENT_OTHER", "Other children and special"),
];
