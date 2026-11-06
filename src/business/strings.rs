pub(crate) const BUSINESS_STRINGS: &[(&str, &str)] = &[
    (
        "DE_MINIMIS_ITEM_ABOVE_LIMIT",
        "The largest invoice or item exceeds the $2,500 limit, or $5,000 with an applicable financial statement. Enter it under its ordinary expense or asset treatment.",
    ),
    (
        "BUSINESS_DEPLETION_NEEDS_A_DEPOSIT",
        "Schedule C depletion requires an oil, gas or mineral deposit; intangible royalties have no depletion deduction.",
    ),
    (
        "NONFARM_OPTIONAL_HISTORY_INVALID",
        "Enter each earlier nonfarm optional-method use once, from1951 through2025; at most four earlier uses.",
    ),
    (
        "WARN_HOUSEHOLD_FUTA_CREDIT_ASSUMED_NONE",
        "{0}: Assumed no FUTA state unemployment credit because the final 2026 credit-reduction rate for {1} is unknown. Enter the official final rate when available; your tax may be lower.",
    ),
    ("DISPLAY_2106_JOB", "Your job"),
    ("DISPLAY_2106_EXPENSES", "Expenses and reimbursements"),
    ("DISPLAY_2106_VEHICLES", "Vehicles"),
    (
        "NOTE_HOUSEHOLD_PAYROLL_SCHEDULE_H_TAXES_ENTER_SCHEDULE_2_NOT_YOUR_PER_3D92019D",
        "Household payroll: Schedule H taxes enter Schedule 2, not your personal withholding credit. Include eligible care or medical employment costs in their expense records only once; do not automatically deduct all wages or taxes. Separate W-2 reporting and payroll filing obligations remain outside this prototype.",
    ),
    (
        "ISSUE_SPOUSE_MOVE_REQUIRES_A_JOINT_RETURN",
        "A spouse's moving expenses require a joint return.",
    ),
    (
        "ISSUE_CONFIRM_COMPLETE_PAYROLL_RECORDS_AND_ONE_ELIGIBLE_EMPLOYER_RECORD_P_8E42C894",
        "Confirm complete payroll records and one eligible employer record per filer; unresolved payroll exceptions need their own treatment.",
    ),
    (
        "ISSUE_CONFIRM_THE_STATE_PAYROLL_NOTICE_EMPLOYER_LIABILITY_AND_DEPOSITED_E_CFDFB10A",
        "Confirm the state payroll notice, employer liability and deposited employee taxes; reconcile withholding with the covered wages and annual limits.",
    ),
    (
        "ISSUE_ENTER_UNEMPLOYMENT_RECORDS_AND_THE_FINAL_CURRENT_YEAR_CREDIT_REDUCTION_RATE_07BEA69A",
        "Enter unemployment records and the final 2026 credit-reduction rate for each state covering FUTA wages.",
    ),
    (
        "ISSUE_RESOLVE_THIS_PARTNERSHIP_S_REGULAR_SE_EARNINGS_AND_K_1_BOX_14_B_C_G_E76F6153",
        "Resolve this partnership's regular SE earnings and K-1 box 14 B/C gross-income category. Ordinary limited-partner income is not optional-method gross income; include entered guaranteed service payments in gross income.",
    ),
    (
        "ISSUE_STATE_DEPOSITED_EMPLOYEE_PAYROLL_TAX_CANNOT_EXCEED_COMBINED_SCHEDUL_F2437532",
        "State-deposited employee payroll tax cannot exceed combined Schedule H payroll tax; reconcile the notice before completing this return.",
    ),
    (
        "GAMBLING_BUSINESS_WITHHOLDING_ABOVE_WINNINGS",
        "W-2G withholding cannot exceed the gross winnings on line 1.",
    ),
    (
        "EMPLOYER_CREDITS_NEED_PAYROLL",
        "Employer credits need this business's employee W-2s under Employees' W-2 wages.",
    ),
    (
        "EMPLOYER_CREDIT_EMPLOYEE_NOT_ON_PAYROLL",
        "Employer credits list only employees whose W-2s are under this business's Employees' W-2 wages. Choose each employee's W-2 from that list.",
    ),
    (
        "MILEAGE_ABOVE_TOTAL_MILES",
        "Business miles plus commuting miles cannot exceed all miles driven in 2026.",
    ),
    (
        "EMPLOYEE_EQUIPMENT_RESERVIST_TRAVEL_SOURCE",
        "A reservist's equipment recovery needs its actual connection to eligible overnight duty travel more than 100 miles from home. Ordinary job use does not establish that travel source; retain the property records.",
    ),
    (
        "EMPLOYEE_EQUIPMENT_SECTION179_SOURCE",
        "This employee equipment retains its actual section 179 election and carry. The employee source must be coordinated with the return-wide section 179 pool before a complete estimate is available.",
    ),
    (
        "EMPLOYEE_EQUIPMENT_RECOVERY_SOURCE",
        "This employee equipment needs its actual original recovery, credit or disposition source before a complete estimate is available.",
    ),
    (
        "ASSET_MACRS_SOURCE_QUESTIONS_UPDATED",
        "Record {0}, property {1}: recovery source questions changed with your current owner, date, class or election answers. Review the source questions and save again. None of these edits has been saved yet.",
    ),
    (
        "EMPLOYEE_MACRS_SOURCE_QUESTIONS_UPDATED",
        "Form 2106 {0}, equipment {1}: recovery source questions changed with your current owner, date, class or election answers. Review the source questions and save again. None of these edits has been saved yet.",
    ),
    ("ASSET_ACQUIRED_AFTER_PLACED", "The date bought cannot be after the date placed in service."),
    (
        "ASSET_DISPOSED_BEFORE_PLACED",
        "The date sold or stopped using must be after the date placed in service.",
    ),
    (
        "ASSET_179_ABOVE_BUSINESS_COST",
        "The section 179 amount cannot exceed the business share of the cost.",
    ),
    (
        "ASSET_NEW_PLACED_IN_2026",
        "Property placed in service in 2026 needs a 2026 date placed in service.",
    ),
    (
        "ASSET_OLD_PLACED_BEFORE_2026",
        "Property placed in service before 2026 needs a date placed in service before 2026.",
    ),
    (
        "EQUIPMENT_RENT_NO_SECTION_179",
        "Section 179 is not available for equipment rented out outside a business; choose bonus or regular depreciation.",
    ),
    (
        "VEHICLE_EARLIER_USE_ONLY_AFTER_USE_FELL",
        "Enter the earlier business use only for a vehicle placed in service before 2026 whose business use this year is 50% or less.",
    ),
    // Form 2106 (employee_expenses.rs): the page's client rules with their server twins.
    ("EMPLOYEE_EXPENSES_EMPLOYER_NEEDED", "Choose at least one W-2 employer for this job."),
    ("EMPLOYEE_EXPENSES_EMPLOYER_TWICE", "Each employer is chosen once."),
    (
        "RESERVIST_VEHICLE_STANDARD_MILEAGE",
        "A reservist's vehicle is deducted at the standard mileage rate; choose Standard mileage rate.",
    ),
    (
        "EMPLOYEE_VEHICLE_NO_INTEREST_OR_TAX",
        "Car-loan interest and personal property tax are not deductible on Form 2106, and recapture after a vehicle's business use falls to 50% or less is not calculated for an employee; leave them at zero and the earlier business use empty.",
    ),
    (
        "MESSAGE_EMPLOYER_MUST_BE_YOUR_W_2",
        "Each employer must be a W-2 of the person whose form this is.",
    ),
    (
        "MESSAGE_EMPLOYER_CHOSEN_ON_ONE_FORM_2106_ONLY",
        "Choose each W-2 on one Form 2106 only; its code L reimbursements reduce one job's expenses.",
    ),
    (
        "MESSAGE_CHECK_THE_FORM_2106_AMOUNTS_AND_VEHICLES",
        "Check the Form 2106 amounts and vehicles: occupation, amounts within the supported limits, and at most 10 vehicles.",
    ),
    (
        "ISSUE_EMPLOYEE_EXPENSES_EMPLOYER_MISSING",
        "A W-2 chosen for this Form 2106 is no longer on the return. Choose this job's W-2 again.",
    ),
    (
        "ISSUE_EMPLOYEE_VEHICLE_ACTUAL_EXPENSES_SCOPE",
        "Form 2106 actual vehicle expenses need an owned 5-year passenger automobile placed in service in 2025 or 2026, or a vehicle outside the passenger-automobile limit, with the IRS table percentages and no section 179 carryforward. Earlier passenger automobiles, a 150% declining-balance history, a lease, an employer-provided vehicle and a personal-use conversion need separate rules (Form 2106 Part II).",
    ),
    (
        "WARN_EMPLOYEE_VEHICLE_179_LEFT_OUT",
        "Form 2106 for {0}: section 179 is not calculated for an employee vehicle, so the amount you elected is left out and the vehicle takes its bonus or regular depreciation instead.",
    ),
    (
        "WARN_PERFORMING_ARTIST_EMPLOYERS",
        "Form 2106 for {0}: a qualified performing artist performed for at least two employers who each paid $200 or more. Your chosen W-2s do not, so the {1,number,currency} of expenses is left out of Schedule 1 line 12.",
    ),
    (
        "WARN_PERFORMING_ARTIST_TEN_PERCENT",
        "Form 2106 for {0}: a qualified performing artist's expenses must be more than 10% of the pay in box 1 of the chosen W-2s. They are not, so the {1,number,currency} of expenses is left out of Schedule 1 line 12.",
    ),
    (
        "WARN_PERFORMING_ARTIST_AGI",
        "Form 2106 for {0}: a qualified performing artist's adjusted gross income before this deduction must be $16,000 or less (combined on a joint return). It is more, so the {1,number,currency} of expenses is left out of Schedule 1 line 12.",
    ),
    (
        "WARN_PERFORMING_ARTIST_SEPARATE_RETURN",
        "Form 2106 for {0}: a married qualified performing artist must file jointly unless the spouses lived apart all year. This separate return does not qualify, so the {1,number,currency} of expenses is left out of Schedule 1 line 12.",
    ),
    (
        "NOTE_EMPLOYEE_EXPENSES_EXCESS_REIMBURSEMENT",
        "Form 2106 for {0}: your employer's reimbursements exceeded your expenses by {1,number,currency}. The excess is wage income on Form 1040 line 1a; correct the W-2 amounts or the expenses if this is wrong.",
    ),
    ("ASSET_CREDIT_ABOVE_COST", "A credit cannot reduce the cost by more than the cost."),
    ("MOVING_ARRIVAL_OUTSIDE_CURRENT_YEAR", "Enter the 2026 date you arrived at your new home."),
    (
        "MOVING_ENTER_AN_EXPENSE",
        "Enter what you paid to move your household goods, or the travel to your new home.",
    ),
    ("DISPLAY_BUSINESS_HEADER", "Business identity and filing"),
    (
        "BUSINESS_MEAL_KIND_ONCE",
        "Enter each meal kind once, at its full cost before the deduction limit.",
    ),
    ("DISPLAY_EMPLOYER_CREDITS", "Employer credits"),
    (
        "REHABILITATION_NEEDS_A_BUILDING_ROW",
        "The rehabilitation credit belongs on a 39-year building row placed in service in 2018 or later that holds only the qualified rehabilitation expenditures.",
    ),
    (
        "ENERGY_CREDIT_NEEDS_NEW_5_YEAR_PROPERTY",
        "The 2026 energy credit belongs on 5-year equipment placed in service in 2026.",
    ),
    (
        "ENERGY_CONSTRUCTION_AFTER_PLACED",
        "Construction cannot begin after the date the property was placed in service.",
    ),
    (
        "EARLIER_ENERGY_NEEDS_CONTINUING_5_YEAR_PROPERTY",
        "An earlier energy credit belongs on 5-year equipment placed in service before 2026.",
    ),
    (
        "EQUIPMENT_RENT_NO_INVESTMENT_CREDIT",
        "The investment credit for equipment you rent out outside a business is not calculated.",
    ),
    (
        "PARTIAL_DISPOSITION_NEEDS_A_BUILDING_ROW",
        "A partial disposition belongs on a 39-year building row.",
    ),
    (
        "ASSET_EARLIER_USE_ONLY_AFTER_USE_FELL",
        "The earlier business use applies only to an asset placed in service before 2026 whose business use is 50% or less this year.",
    ),
    ("DISPLAY_BUSINESS_RECEIPTS", "Receipts and expenses"),
    ("DISPLAY_BUSINESS_ASSETS", "Vehicles, assets and home office"),
    ("DISPLAY_C_LINES_8_16", "Lines 8–16"),
    ("DISPLAY_C_LINES_17_22", "Lines 17–22"),
    ("DISPLAY_C_LINES_23_27", "Lines 23–27"),
    ("DISPLAY_VEHICLE_ACTUAL_COST", "Cost and miles"),
    ("DISPLAY_VEHICLE_ACTUAL_OPERATING", "Operating costs and earlier use"),
    ("DISPLAY_VEHICLE_ACTUAL_HISTORY", "Depreciation history and disposal"),
];
