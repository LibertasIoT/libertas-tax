pub(crate) const INCOME_STRINGS: &[(&str, &str)] = &[
    (
        "BUSINESS_RETURNS_ABOVE_RECEIPTS",
        "Returns and allowances cannot exceed the business receipts entered here.",
    ),
    (
        "NOTE_DEPLETION_65_PERCENT_LIMIT",
        "The oil and gas depletion limit uses taxable income before depletion and QBI. For this business, the half-SE deduction in that limit is figured before depletion, which can conservatively reduce the deduction; keep the carryover.",
    ),
    (
        "WARN_CLERGY_RECORD_MISSING",
        "Ministerial wages are included in self-employment tax. Enter the clergy record to supply any housing or approved exemption.",
    ),
    (
        "ROYALTY_UNITS_SOLD_WITHIN_RECOVERABLE",
        "Units sold cannot exceed the recoverable units at the start of the year.",
    ),
    (
        "ROYALTY_INTANGIBLE_NO_DEPLETION",
        "A copyright, patent or other intangible royalty does not take depletion.",
    ),
    (
        "ROYALTY_ADVANCE_WITHIN_RECEIPTS",
        "Lease bonuses and advance royalties cannot exceed the total royalties, including linked Forms 1099-MISC.",
    ),
    (
        "OTHER_INCOME_WITHHOLDING_ABOVE_AMOUNT",
        "Federal income tax withheld cannot exceed the amount received.",
    ),
    (
        "NEC_WITHHOLDING_ABOVE_PAY",
        "Federal income tax withheld cannot exceed the payments on the form.",
    ),
    (
        "NEC_EXCESS_ABOVE_TOTAL_PAY",
        "Box 3 excess golden parachute payments cannot exceed the total compensation in box 1a.",
    ),
    (
        "MISC_WITHHOLDING_ABOVE_INCOME",
        "Federal income tax withheld cannot exceed the income boxes of the form.",
    ),
    (
        "MISC_CLIENT_FUNDS_ABOVE_PROCEEDS",
        "The part paid on to clients cannot exceed the gross proceeds in box 10.",
    ),
    ("K_PARTS_NOT_GROSS", "The parts must add up to the gross amount in box 1a."),
    (
        "K_WITHHOLDING_ABOVE_GROSS",
        "Federal income tax withheld cannot exceed the gross amount in box 1a.",
    ),
    ("K_REFUNDS_ABOVE_PART", "Refunds cannot exceed the part they come out of."),
    (
        "K_REFUNDS_ONLY_ON_INCOME",
        "Refunds apply only to a part that is income: a business, a rental or a hobby.",
    ),
    (
        "W2_SCHEDULE_1_PAY_ABOVE_WAGES",
        "The pension, inmate pay and excluded Medicaid waiver payments moved to Schedule 1 cannot exceed box 1.",
    ),
    (
        "NEC_EMPLOYEE_PAY_HAS_GOLDEN_PARACHUTE",
        "Excess golden parachute payments in box 3 carry a 20% excise tax that employee pay on Form 8919 does not figure; this form cannot be entered as employee pay.",
    ),
    (
        "FORM_1099_USE_NOT_RECOVERY",
        "A Form 1099 does not report a refund of an earlier year's deduction; enter that as its own Other income item.",
    ),
    (
        "ISSUE_FORM_1099_DESTINATION_MISSING",
        "Enter the business that recovered this previously deducted expense, then select it in the tax-refund record.",
    ),
    (
        "ISSUE_FORM_1099_DESTINATION_NOT_CHOSEN",
        "Select the business that recovered this previously deducted expense in the tax-refund record.",
    ),
    (
        "ISSUE_RECOVERY_YEAR_TWICE",
        "Combine every refund or reimbursement of the same year's itemized deductions into one Other income recovery: Publication 525 Worksheet 2 is figured once for each year.",
    ),
    (
        "ISSUE_RECOVERY_WITH_MULTIPLE_YEAR_REFUND",
        "A state or local tax refund allocated across several prior years cannot be figured with another recovery of the same year here; combine them on a review of that year's Worksheet 2.",
    ),
    (
        "ESPP_ACQUISITION_LINK_NEEDED",
        "Choose the Form 3922 for this employee and these sold shares.",
    ),
    (
        "ESPP_RECORDS_GRANT_PRICE_NEEDED",
        "Supply the exercise price per share as if exercised on the grant date from the plan records of these ESPP shares.",
    ),
    (
        "ESPP_RECORD_GRANT_PRICE_NEEDED",
        "The stock plan record does not show the purchase price as if bought on the grant date. Supply it from the plan's offering records.",
    ),
    (
        "ESPP_SOLD_ABOVE_TRANSFERRED",
        "The shares sold from this acquisition exceed the shares transferred on Form 3922.",
    ),
    (
        "ESPP_LOT_REVIEW_NEEDED",
        "Review the ESPP acquisition and sale records; this lot does not meet the existing calculation limits.",
    ),
    (
        "ESPP_TRANSFERRED_BEFORE_EXERCISED",
        "Form 3922 box 7 cannot be before the exercise in box 2.",
    ),
    ("DISPLAY_ISO_1099B", "1099-B"),
    ("DISPLAY_ISO_3921", "Form 3921"),
    ("DISPLAY_SALE_STOCK_PLAN", "Stock plan records"),
    ("DISPLAY_MISC_WITHHELD", "Withholding and state boxes"),
    (
        "ISSUE_COMBINED_STOCK_COMPENSATION_MARKED_AS_ALREADY_REPORTED_CANNOT_EXCEE_0E5F6C00",
        "Combined stock compensation marked as already reported cannot exceed this filer's entered W-2 box 1 wages. Reconcile employer reporting before calculating.",
    ),
    ("W2_WITHHOLDING_ABOVE_WAGES", "Box 2 cannot exceed box 1 wages."),
    (
        "W2_SOCIAL_TAX_ABOVE_WAGES",
        "Box 4 Social Security tax withheld cannot exceed the combined box 3 wages and box 7 tips.",
    ),
    ("W2_MEDICARE_TAX_ABOVE_WAGES", "Box 6 cannot exceed box 5 Medicare wages."),
    (
        "W2_SOCIAL_WAGES_AND_TIPS_ABOVE_BASE",
        "Box 3 plus box 7 cannot exceed the $184,500 Social Security wage base.",
    ),
    ("W2_BOX12_CODE_TWICE", "Enter each box 12 code only once."),
    (
        "FORM_8919_CODE_H_NEEDS_1099",
        "Reason H applies only when the same firm gave you a Form 1099 as well as a W-2; answer Yes to receiving a Form 1099.",
    ),
    ("FORM_8919_WITHHOLDING_ABOVE_PAY", "Withholding cannot exceed the pay."),
    (
        "EMPLOYEE_RAILROAD_SOURCE_INVALID",
        "Enter a nonnegative actual railroad source amount, or identify it as unavailable or not reported.",
    ),
    (
        "JURY_REMITTED_ABOVE_RECEIVED",
        "Pay you turned over to your employer cannot exceed the jury pay you received.",
    ),
    (
        "PRIZE_WITHHOLDING_ABOVE_AWARD",
        "Box 4 cannot exceed box 3 plus the fair market value of goods or services won.",
    ),
    (
        "ISO_SOLD_BEFORE_EXERCISED",
        "ISO shares sold: the date sold (1099-B box 1c) cannot be before the date exercised (Form 3921 box 2).",
    ),
    ("ISO_SALE_NOT_IN_RETURN_YEAR", "ISO shares sold: enter a sale made in 2026 (1099-B box 1c)."),
    (
        "EMPLOYEE_STOCK_SOLD_BEFORE_TAXED",
        "Employee stock shares sold: the date sold (1099-B box 1c) cannot be before the date the shares were taxed as wages.",
    ),
    (
        "EMPLOYEE_STOCK_SALE_NOT_IN_RETURN_YEAR",
        "Employee stock shares sold: enter a sale made in 2026 (1099-B box 1c).",
    ),
    (
        "ESPP_EXERCISED_BEFORE_GRANTED",
        "ESPP shares sold: the date exercised (Form 3922 box 2) cannot be before the date granted (box 1).",
    ),
    (
        "ESPP_SOLD_BEFORE_EXERCISED",
        "ESPP shares sold: the date sold (1099-B box 1c) cannot be before the date exercised (Form 3922 box 2).",
    ),
    (
        "ESPP_SALE_NOT_IN_RETURN_YEAR",
        "ESPP shares sold: enter a sale made in 2026 (1099-B box 1c).",
    ),
    ("EMPLOYEE_EXPENSES_EMPLOYER_CHOICE", "{0} — {1}"),
    ("DISPLAY_W2_BOXES_1_6", "Boxes 1–6"),
    ("DISPLAY_W2_BOXES_7_10", "Boxes 7–10 tips and care benefits"),
    ("DISPLAY_W2_BOXES_A_F", "Boxes a–f employer and employee"),
    ("DISPLAY_W2_EMPLOYER_NAME", "Employer name"),
    ("DISPLAY_W2_EMPLOYEE_NAME", "Employee name"),
    ("DISPLAY_FORM3921_TRANSFEROR", "Company that transferred the shares"),
    ("DISPLAY_FORM3921_EMPLOYEE", "Employee"),
    ("DISPLAY_FORM3921_CORPORATION", "Corporation whose stock was transferred"),
    ("DISPLAY_W2_BOXES_11_14", "Boxes 11–14"),
    ("DISPLAY_W2_BOXES_15_20", "Boxes 15–20 state and local"),
    ("W2_ABROAD_ABOVE_WAGES", "Pay for work abroad cannot exceed box 1."),
    (
        "ISSUE_STATUTORY_BUSINESS_HAS_OTHER_INCOME",
        "Select this statutory job's Schedule C with expenses only, no other income or owner health and retirement deductions.",
    ),
    (
        "ISSUE_STATUTORY_BUSINESS_OTHER_OWNER",
        "The statutory employee W-2 and its Schedule C must belong to the same person.",
    ),
    (
        "WARN_STATUTORY_PAY_LEFT_OUT",
        "Statutory pay is left out with its unsupported Schedule C; withholding and payroll boxes are retained.",
    ),
    (
        "NOTE_HOUSEHOLD_PAY_W2_DUE",
        "An employer paying $3,000 or more in cash household wages in 2026 generally owes a W-2 and payroll taxes. Obtain and enter the W-2 instead of duplicating the pay.",
    ),
    (
        "W2_STATUTORY_ABROAD_USE_BUSINESS",
        "For statutory employee pay abroad, leave the W-2 foreign-pay amount zero and select a saved Schedule C, even with no expenses. Record where its services were performed on that business so its income and related deductions are counted once.",
    ),
    ("DISPLAY_W2_TIPS", "Tips"),
    ("DISPLAY_W2_PAY_TREATMENT", "Pay treatment"),
    ("DISPLAY_W2_PERSONAL", "Personal records"),
];
