pub(crate) const FORM_STRINGS: &[(&str, &str)] = &[
    (
        "NOTE_CARE_NO_PROVIDER",
        "Form 2441 Part I line 1a: none. No qualifying-person expenses are available to exclude the dependent-care benefits.",
    ),
    (
        "WARN_CARE_PLAN_NOT_ENTERED",
        "Dependent-care benefits are taxable until the plan limit and benefit answers are saved on Child and dependent care.",
    ),
    (
        "WARN_CARE_SPOUSE_EARNINGS_ASSUMED_ZERO",
        "Assumed your spouse's earnings are unknown, so they count as zero and all your employer care benefits are taxable; if they are known, your tax may be lower.",
    ),
    (
        "NOTE_FORM_8862_PART_III_LISTS_FOUR_CHILDREN_AND_FOUR_OTHER_DEPENDENTS_7C3CBA57",
        "Form 8862 Part III lists four children and four other dependents; the rest need an attached statement answering the same questions.",
    ),
    (
        "NOTE_FORM_8862_PART_IV_LISTS_THREE_STUDENTS_THE_REST_NEED_AN_ATTACHED_B8DCB35A",
        "Form 8862 Part IV lists three students; the rest need an attached statement answering the same questions.",
    ),
    (
        "NOTE_FORM_8862_SECTION_A_LISTS_THE_THREE_SCHEDULE_EIC_CHILDREN_THE_OT_3C959748",
        "Form 8862 Section A lists the three Schedule EIC children; the other qualifying children are not listed.",
    ),
    (
        "NOTE_8862_EIC_BANNED",
        "Form 8862: the earned income credit is not allowed for 2026 because the {0}-year ban from the {1,number,0} disallowance runs through tax year {2,number,0}, so this estimate leaves it out. To appeal the ban you would claim the credit on a mailed return with Form 8862 attached; the IRS rejects an e-filed claim during the ban.",
    ),
    (
        "NOTE_8862_CHILD_CREDITS_BANNED",
        "Form 8862: the child tax credit, additional child tax credit and credit for other dependents are not allowed for 2026 because the {0}-year ban from the {1,number,0} disallowance runs through tax year {2,number,0}, so this estimate leaves them out; the people stay your dependents. To appeal the ban you would claim the credits on a mailed return with Form 8862 attached; the IRS rejects an e-filed claim during the ban.",
    ),
    (
        "NOTE_8862_EIC_MATH_ERROR",
        "Form 8862 is not needed for the earned income credit: the earlier reduction was a math or clerical error only.",
    ),
    (
        "NOTE_8862_EIC_REINSTATED",
        "Form 8862 is not needed for the earned income credit: the credit was allowed again after Form 8862 and has not been disallowed since.",
    ),
    (
        "NOTE_8862_EIC_BEFORE_REQUIREMENT",
        "Form 8862 is not needed for the earned income credit: the disallowed year is before the requirement began.",
    ),
    (
        "NOTE_8862_EIC_NOT_CLAIMED",
        "Form 8862 is not needed for the earned income credit: the return does not claim it.",
    ),
    (
        "NOTE_8862_EIC_CHILDLESS_EXCEPTION",
        "Form 8862 is not needed for the earned income credit: it is claimed without a qualifying child and the only earlier finding was that a listed child was not a qualifying child.",
    ),
    (
        "NOTE_8862_CHILD_MATH_ERROR",
        "Form 8862 is not needed for the child tax credit, additional child tax credit or credit for other dependents: the earlier reduction was a math or clerical error only.",
    ),
    (
        "NOTE_8862_CHILD_REINSTATED",
        "Form 8862 is not needed for the child tax credit, additional child tax credit or credit for other dependents: the credit was allowed again after Form 8862 and has not been disallowed since.",
    ),
    (
        "NOTE_8862_CHILD_BEFORE_REQUIREMENT",
        "Form 8862 is not needed for the child tax credit, additional child tax credit or credit for other dependents: the disallowed year is before the requirement began.",
    ),
    (
        "NOTE_8862_CHILD_NOT_CLAIMED",
        "Form 8862 is not needed for the child tax credit, additional child tax credit or credit for other dependents: the return does not claim it.",
    ),
    (
        "NOTE_8862_EDUCATION_MATH_ERROR",
        "Form 8862 is not needed for the American opportunity tax credit: the earlier reduction was a math or clerical error only.",
    ),
    (
        "NOTE_8862_EDUCATION_REINSTATED",
        "Form 8862 is not needed for the American opportunity tax credit: the credit was allowed again after Form 8862 and has not been disallowed since.",
    ),
    (
        "NOTE_8862_EDUCATION_BEFORE_REQUIREMENT",
        "Form 8862 is not needed for the American opportunity tax credit: the disallowed year is before the requirement began.",
    ),
    (
        "NOTE_8862_EDUCATION_NOT_CLAIMED",
        "Form 8862 is not needed for the American opportunity tax credit: the return does not claim it.",
    ),
    (
        "WARN_8862_HOME_DAYS_TAXPAYER",
        "Form 8862 line 9a needs the number of days in 2026 that your main home was in the United States; the estimate doesn't depend on it, but enter it on the Form 8862 credit before filing.",
    ),
    (
        "WARN_8862_HOME_DAYS_SPOUSE",
        "Form 8862 line 9b needs the number of days in 2026 that your spouse's main home was in the United States; the estimate doesn't depend on it, but enter it on the Form 8862 credit before filing.",
    ),
    (
        "ISSUE_8862_HOME_DAYS_CONTRADICTION_TAXPAYER",
        "Form 8862 line 9a: {0} days contradicts the answer that your main home was in the United States for more than half of 2026; correct one of them.",
    ),
    (
        "ISSUE_8862_HOME_DAYS_CONTRADICTION_SPOUSE",
        "Form 8862 line 9b: {0} days contradicts the answer that your spouse's main home was in the United States for more than half of 2026; correct one of them.",
    ),
    (
        "ISSUE_1040ES_FINAL_RETURN",
        "Form 1040-ES: a 2027 estimated tax is not figured for a final return.",
    ),
    (
        "ISSUE_1040ES_TAX_WORKSHEET",
        "Form 1040-ES line 4: this return's tax is not the Tax Rate Schedule or Qualified Dividends and Capital Gain Tax Worksheet tax (Form 8615, Schedule J or the Form 2555 exclusion applies); the Pub. 505 worksheet for that tax is not calculated.",
    ),
    (
        "ISSUE_1040ES_AMT_CHANGED_INCOME",
        "Form 1040-ES line 5: this return has alternative minimum tax, and Form 6251 for 2027 is not refigured for a changed income or deduction; choose the estimate from this return as it is.",
    ),
    (
        "NOTE_1040ES_UNPUBLISHED_PACKAGE",
        "Form 1040-ES for {0,number,0}: the IRS has not published the {0,number,0} package, so the worksheet uses the {1,number,0} Tax Rate Schedules and this return's {1,number,0} deductions, credits and other taxes as the estimate. Refigure it with the {0,number,0} Form 1040-ES when it is published.",
    ),
    (
        "NOTE_1040ES_NO_REQUIRED_PAYMENT",
        "Form 1040-ES line 14: you are not required to make 2027 estimated tax payments (the required annual payment is covered by expected withholding, or the expected balance is under $1,000). You may still owe tax when you file.",
    ),
    (
        "NOTE_1040ES_FARMER_INSTALLMENT",
        "Form 1040-ES line 15 (farmers and fishermen): pay the {0,number,currency} required annual payment in one installment by January 18, 2028, or file your 2027 return and pay the tax in full by March 1, 2028. Subtract any 2026 overpayment you apply to 2027.",
    ),
    (
        "NOTE_1040ES_QUARTERLY_INSTALLMENT",
        "Form 1040-ES line 15: pay {0,number,currency} by each of April 15, 2027, June 15, 2027, September 15, 2027 and January 18, 2028 (January 15, 2028 is a Saturday and January 17 a legal holiday). Subtract any 2026 overpayment you apply to the first installment; you may instead pay the whole amount by April 15, 2027.",
    ),
    (
        "NOTE_1040ES_OMITTED_REPAYMENTS",
        "Form 1040-ES: this return repays {0,number,currency} of excess advance premium tax credit or transferred clean-vehicle credit on Schedule 2 Part I; the Estimated Tax Worksheet's lines 5, 6 and 10 do not list those repayments, so they are not in the 2027 estimate. Add them yourself if you expect them again.",
    ),
    (
        "NOTE_1040ES_ANNUALIZED_NOT_CALCULATED",
        "Form 1040-ES line 15 divides the required annual payment into four equal installments; the annualized income installment method for 2027 (Pub. 505 Worksheet 2-7) is not calculated.",
    ),
    (
        "ISSUE_A_CHURCH_EMPLOYEE_W_2_MUST_SHOW_ZERO_SOCIAL_SECURITY_AND_MEDICARE_W_F52B07D4",
        "A church employee or ministerial W-2 must show zero Social Security and Medicare wages (boxes 3 and 5); box 1 is reported on Schedule SE line 5a for church employees or line 2 for ministerial services.",
    ),
    (
        "ISSUE_CONFIRM_THE_DISABILITY_RETIREMENT_AGE_AND_MEDICAL_STATEMENT_REQUIRE_978C4752",
        "Confirm the disability-retirement, age and medical-statement requirements for these Schedule R wages.",
    ),
    (
        "ISSUE_EXCESS_SOCIAL_SECURITY_WITHHOLDING_BY_ONE_EMPLOYER_REQUIRES_EMPLOYE_35C66F28",
        "Excess Social Security withholding by one employer requires employer correction, not the multiple-employer credit.",
    ),
    (
        "ISSUE_MEDICARE_WITHHOLDING_BELOW_ORDINARY_PAYROLL_TAX_REQUIRES_SPECIAL_PA_3993480B",
        "Medicare withholding below ordinary payroll tax requires special payroll review.",
    ),
    (
        "ISSUE_RECONCILE_THIS_EMPLOYER_S_QUALIFIED_FORM_4137_COLUMN_1_C_CASH_CHARG_8A2B870D",
        "Reconcile this employer's qualified Form 4137 column 1(c) cash/charged tips with its employee-tip records; Form 4137 cannot supply a deduction without the corresponding wage-income source.",
    ),
    (
        "ISSUE_RECONCILE_W_2_BOX_12_SOCIAL_SECURITY_ORDINARY_MEDICARE_AMOUNTS_WITH_6BFF7BFD",
        "Reconcile W-2 box 12 Social Security/ordinary Medicare amounts with the covered wage boxes; exclude railroad and Additional Medicare tax amounts.",
    ),
    (
        "NOTE_1040ES_CREDIT_ALREADY_APPLIED",
        "The planned line 36 credit is already applied in the remaining installment schedule. Do not subtract it again. The regular due dates are April 15, June 15 and September 15, 2027, and January 18, 2028; farmers and fishermen have one installment on January 18, 2028 or may file and pay in full by March 1, 2028.",
    ),
    (
        "NOTE_1040ES_APPLIED_INSTALLMENT",
        "2027 planned installment {0}: {1,number,currency} before applied credit, {2,number,currency} planned credit used, {3,number,currency} remaining cash payment. This forecast does not establish payment or filing.",
    ),
    (
        "NOTE_1040ES_CREDIT_REMAINDER",
        "Planned applied credit remaining after this estimated-payment schedule: {0,number,currency}. It is not additional current-year withholding or a payment already made.",
    ),
    (
        "NOTE_RAILROAD_TIPS_REPORT_TO_EMPLOYER",
        "Railroad tips are income on Form 1040 line 1c. Contact the railroad employer to report them and resolve their RRTA tax; Form 4137 does not apply.",
    ),
];
