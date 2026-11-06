pub(crate) const PAYMENT_STRINGS: &[(&str, &str)] = &[
    ("DOCUMENT_TRUMP_RETURN_DATE", "When was this corrective distribution received?"),
    (
        "DOCUMENT_TRUMP_RETURN_DEADLINE",
        "What filing deadline, including any extension or relief, applied to this correction?",
    ),
    ("COVERDELL_TOPIC_TWICE", "Enter one Coverdell topic with one beneficiary record per filer."),
    (
        "REFUND_PROPERTY_ABOVE_OTHER_TAXES",
        "Real estate and personal property taxes refunded cannot exceed the taxes other than income tax that you deducted that year.",
    ),
    (
        "WARN_PRIOR_AGI_ASSUMED_HIGH",
        "Assumed your 2025 AGI was above $150,000, so the prior-year safe harbor is 110% of your 2025 tax; if it was $150,000 or less, your penalty may be lower.",
    ),
    (
        "ISSUE_REFUND_MULTI_YEAR_ALLOCATION_INCOMPLETE",
        "The multi-year refund allocation is incomplete.",
    ),
    (
        "ISSUE_REFUND_PRIOR_CURRENT_ALLOCATION_INCOMPLETE",
        "The prior/current-year refund allocation is incomplete.",
    ),
    (
        "ISSUE_REFUND_TAX_BENEFIT_WORKSHEET_NEEDED",
        "This refund needs the Publication 525 tax-benefit recovery worksheet, including prior credit, AMT or other recovery coordination, and the recovery of a prior Form 8960 line 9b deduction on Form 8960 line 7.",
    ),
    (
        "ISSUE_REFUND_EXCEEDS_PRIOR_TAX_PAYMENT",
        "The refund exceeds the state/local income tax payment recorded for this deduction year. Separate other-year taxes, interest, credits or other recovery sources.",
    ),
    (
        "ISSUE_REFUND_PRIOR_YEARS_UNRECONCILED",
        "Reconcile this one refund to 2–8 distinct prior federal payment and deduction years and their retained state and federal records.",
    ),
    (
        "ISSUE_REFUND_EXCEEDS_PRIOR_YEAR_PAYMENTS",
        "The refund exceeds payments in the recorded prior years of the state return; separate interest or other sources.",
    ),
    (
        "ISSUE_REFUND_PRIOR_CURRENT_UNRECONCILED",
        "Reconcile this one refund, its prior federal Schedule A deductions, and its 2026 income-tax payment to the retained state and federal records.",
    ),
    (
        "ISSUE_REFUND_EXCEEDS_PRIOR_CURRENT_PAYMENTS",
        "The refund exceeds payments in the recorded prior years and 2026 of the state return; separate interest or other sources.",
    ),
    ("DISPLAY_PERIOD_MARCH", "Jan–Mar"),
    ("DISPLAY_PERIOD_MAY", "Jan–May"),
    ("DISPLAY_PERIOD_AUGUST", "Jan–Aug"),
    ("DISPLAY_PERIOD_YEAR", "Full year"),
    (
        "ISSUE_2210_FARMLAND_ELECTION",
        "The selected section 1062 farmland election needs its applicable net tax liability before Form 2210 line 3 or Form 2210-F line 4 can be calculated.",
    ),
    (
        "ISSUE_2210_AI_FARMLAND_PERIODS",
        "Schedule AI needs the applicable section 1062 farmland tax liability attributable to each income period; the annual amount cannot establish those period amounts.",
    ),
    (
        "ISSUE_2210_FUND_CREDIT_DATE",
        "Form 2439 tax paid by a fund or REIT has no payment date on Form 2210 line 11 or Form 2210-F line 12 in this App.",
    ),
    (
        "ISSUE_2210_JANUARY_PAYMENT_EXCEPTION",
        "Form 2210 line 11 column (d): a balance paid January 16-31, 2027 may be treated as paid on the January 15 due date if you file your return and pay in full by January 31; this App charges interest to the actual payment date instead of applying that rule.",
    ),
    (
        "ISSUE_2210_WITHHOLDING_PERIODS",
        "Form 2210 Part II box D: withholding on the dates actually withheld needs the four period amounts, adding up to Form 1040 line 25d plus excess Social Security.",
    ),
    (
        "ISSUE_2210_SCHEDULE_AI_PERIODS",
        "Form 2210 Part II box C: the annualized income installment method needs Schedule AI's income and deductions for each period.",
    ),
    (
        "ISSUE_2210_AI_ITEMIZED_ROUTE",
        "Schedule AI line 4 takes itemized deductions only when you itemize on your return; enter 0 in every period.",
    ),
    (
        "ISSUE_2210_AI_FULL_YEAR_MATCH",
        "Schedule AI column (d) must match your full-year return: line 1 must equal your adjusted gross income and line 13 your taxable income.",
    ),
    (
        "ISSUE_2210_AI_TAX_WORKSHEET",
        "Schedule AI line 14 is figured here only from the Tax Table or Tax Computation Worksheet; qualified dividends, capital gains, excluded foreign income, Form 8615 or Schedule J need their worksheets refigured for each period, which this App does not do.",
    ),
    (
        "ISSUE_2210_AI_OTHER_TAXES",
        "Schedule AI line 16 needs other taxes figured for each period (such as Additional Medicare Tax, net investment income tax, alternative minimum tax or household employment tax), which this App does not do.",
    ),
    (
        "ISSUE_2210_AI_TWO_SE_FILERS",
        "Schedule AI Part II is completed separately for each spouse with self-employment income; this App completes it for one person.",
    ),
    (
        "ISSUE_2210_AI_SE_FULL_YEAR_MATCH",
        "Schedule AI Part II column (d) must match your full-year self-employment tax: check the full-year net profit and Social Security wages.",
    ),
    (
        "NOTE_2210_WAIVER_REQUESTED",
        "Form 2210 Part II box {0} waiver requested: {1} - {2}; attach the explanation and any documentation; the IRS decides the waiver; the penalty after the amount you ask to waive is shown, and the computed penalty is its own row.",
    ),
    (
        "NOTE_2210F_WAIVER_REQUESTED",
        "Form 2210-F Part I box A waiver requested: {0} - {1}; attach the explanation and any documentation; the IRS decides the waiver; the penalty after the amount you ask to waive is shown, and the computed penalty is its own row.",
    ),
    (
        "NOTE_2210F_PERIOD_ANSWERS_NOT_USED",
        "Farmers and fishers figure the penalty on Form 2210-F, which has one payment due January 15, 2027, so your withholding dates and income by period are not used. Choose evenly for both on Federal payments.",
    ),
    (
        "WAIVER_REASON_RETIREMENT_OR_DISABILITY",
        "retirement after reaching age 62 or disability, with reasonable cause",
    ),
    (
        "WAIVER_REASON_CASUALTY_DISASTER_OR_UNUSUAL",
        "casualty, disaster, or other unusual circumstance",
    ),
    ("WAIVER_REASON_FEDERALLY_DECLARED_DISASTER", "a federally declared disaster"),
    (
        "MESSAGE_AN_EXPECTED_2027_AMOUNT_IS_OUTSIDE_THE_SUPPORTED_RANGE",
        "An expected 2027 amount is outside the supported range.",
    ),
    (
        "MESSAGE_A_PENALTY_WAIVER_REQUEST_NEEDS_AN_EXPLANATION_OF_UP_TO_2_000_CHARACTERS",
        "A penalty waiver request needs a nonblank explanation.",
    ),
    (
        "MESSAGE_A_TRUMP_ACCOUNT_FOR_YOUR_SPOUSE_IS_ENTERED_ONLY_ON_A_JOINT_RETURN",
        "A Trump account for your spouse is entered only on a joint return.",
    ),
    (
        "MESSAGE_CHOOSE_NOT_NEEDED_FOR_2027_FARMING_OR_FISHING_ONLY_WHEN_THIS_RETURN_SHOWS_AT",
        "Choose Not needed for 2027 farming or fishing only when this return shows at least two-thirds of your 2026 gross income from farming or fishing; otherwise answer for 2027.",
    ),
    (
        "MESSAGE_CHOOSE_YES_FOR_2026_ONLY_WHEN_THIS_RETURN_SHOWS_AT_LEAST_TWO_THIRDS_OF_YOUR",
        "Choose Yes, for 2026 only when this return shows at least two-thirds of your 2026 gross income from farming or fishing; otherwise answer for 2025.",
    ),
    (
        "MESSAGE_ENTER_EACH_PERIOD_S_AMOUNTS_WITHIN_THE_SUPPORTED_RANGE",
        "Enter each period's amounts within the supported range.",
    ),
    (
        "MESSAGE_FORM_5329_A_RETURNED_EXCESS_CONTRIBUTION_NEEDS_A_RECEIPT_DATE_FROM_JULY_4_20",
        "Form 5329: a returned excess contribution needs a receipt date from July 4, 2026 through 2028 and a 2026-return due date in 2027 or 2028.",
    ),
    (
        "MESSAGE_FORM_8828_ENTER_A_2026_DISPOSITION_DATE_AN_ORIGINAL_LOAN_CLOSED_AFTER_1990_A",
        "Form 8828: enter a 2026 disposition date, an original loan closed after 1990 and before it, any earlier payoff between the two, and a home improvement loan of at most $15,000.",
    ),
    (
        "MESSAGE_SCHEDULE_H_ENTER_2026_PAYMENT_DATES_TWO_LETTER_STATE_CODES_EACH_2025_QUARTER",
        "Schedule H: enter 2026 payment dates, two-letter state codes, each 2025 quarter 1 through 4 once, each state once and rate periods in 2026 that do not overlap.",
    ),
    (
        "MESSAGE_SCHEDULE_H_YOUR_SPOUSE_S_HOUSEHOLD_EMPLOYMENT_IS_ENTERED_ONLY_ON_A_JOINT_RET",
        "Schedule H: your spouse's household employment is entered only on a joint return.",
    ),
    (
        "PAYMENT_AMOUNT_UNREPRESENTABLE",
        "A native payment, refund or penalty amount cannot be represented exactly. Review the payment sources.",
    ),
    (
        "PAYMENT_WITHHOLDING_UNAVAILABLE",
        "The actual withholding total is not available yet. Complete or correct its source documents before comparing withholding periods.",
    ),
    (
        "MESSAGE_YOUR_SPOUSE_S_SEPARATE_2025_TAX_FORM_2210_BOX_E_APPLIES_ONLY_WHEN_YOU_FILE_J",
        "Your spouse's separate 2025 tax (Form 2210 box E) applies only when you file jointly for 2026.",
    ),
    (
        "PROBLEM_UNDISPLAYABLE_PAYMENTS_RECORD",
        "This return holds a payments record this page cannot show yet. Use a fresh development task.",
    ),
    (
        "INJURED_SPOUSE_EXTENSION_PAYER_REQUIRED",
        "Identify who made the extension payment. Leave the payer empty only when there is no payment.",
    ),
    (
        "FORM_1099G_WITHHOLDING_ABOVE_BENEFITS",
        "Box 4 cannot exceed the payments in boxes 1, 5, 6, 7, 9 and 10.",
    ),
    ("REFUND_WITHHOLDING_ABOVE_REFUND", "Box 4 cannot exceed box 2 refund, credit or offset."),
    ("REFUND_SALT_DEDUCTED_ABOVE_PAID", "Line 5e allowed taxes cannot exceed line 5d taxes paid."),
    ("REFUND_INCOME_TAX_ABOVE_SALT", "Line 5a state income tax cannot exceed line 5d taxes paid."),
    (
        "WITHHOLDING_PERIODS_NOT_TOTAL",
        "The four withholding periods must add up to your total federal income tax withheld.",
    ),
    (
        "FARMER_EVEN_TIMING",
        "Farmers and fishers figure the penalty on Form 2210-F, which has one payment due January 15, 2027: choose evenly for both withholding and income.",
    ),
    (
        "HOUSEHOLD_EMPLOYER_TWICE",
        "Schedule H: enter one household employer for each of you, with all of that person's household employees.",
    ),
    (
        "MORTGAGE_SUBSIDY_HOME_TWICE",
        "Form 8828: give each home its own name and enter each home sale once.",
    ),
    (
        "PAY_CREDIT_RECIPIENT_NEEDED",
        "Choose the taxpayer or spouse account to receive the planned applied credit.",
    ),
    (
        "PAYMENT_APPLIED_CREDIT_NEGATIVE",
        "Enter zero or a positive amount to apply to next year's estimated tax.",
    ),
    (
        "PAY_CREDIT_ABOVE_CAPACITY",
        "The applied credit cannot exceed the available overpayment of {0,number,currency}. Correct the amount; the retained election has not been changed.",
    ),
    ("DISPLAY_PAYMENTS_MADE", "Payments made"),
    ("DISPLAY_PAYMENTS_PENALTY", "Penalty and next year"),
];
